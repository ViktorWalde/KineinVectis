-- O runtime da Kinein dentro do Neovim.
--
-- Três papéis: a contenção (o processo perde o execve, camada 2 do 65 §8), a
-- edição sem modo (65 §6.4 e §12.3) e a ligação com o core, que é dono do
-- lançamento dos servidores, do espelho do buffer e do rascunho. Nada aqui
-- decide caminho de programa nem argumento de servidor: isso vem do core.
local M = {}

local ffi = require('ffi')

ffi.cdef([[
long syscall(long number, ...);
int prctl(int option, unsigned long arg2, unsigned long arg3, unsigned long arg4, unsigned long arg5);
int close(int fd);
char *strerror(int errnum);
typedef struct { uint64_t handled_access_fs; } kinein_landlock_attr;
]])

local SYS_LANDLOCK_CREATE_RULESET = 444
local SYS_LANDLOCK_RESTRICT_SELF = 446
local LANDLOCK_ACCESS_FS_EXECUTE = 1
local LANDLOCK_CREATE_RULESET_VERSION = 1
local LANDLOCK_RESTRICT_SELF_TSYNC = 8
local PR_SET_NO_NEW_PRIVS = 38
local EINVAL = 22

M.containment = { layer2 = false }
M.modal = vim.env.KINEIN_PROVA_MODAL == '1'

local function long(v)
  return ffi.cast('long', v)
end

local function syscall(...)
  local r = tonumber(ffi.C.syscall(...))
  return r, ffi.errno()
end

local function errstr(e)
  return ffi.string(ffi.C.strerror(e))
end

-- Camada 2: o próprio nvim aplica um domínio Landlock que trata o direito de
-- executar e não libera caminho nenhum. A camada 1, do lançador, ainda deixa
-- o binário do nvim e o ld.so; esta fecha os dois, e o ld.so deixa de servir
-- de trampolim para um programa do projeto.
function M.contain()
  if vim.env.KINEIN_PROVA_PLANTIO_SEM_LANDLOCK == '1' then
    M.containment.reason = 'defeito plantado: camada 2 desligada'
    return
  end
  local abi, e = syscall(long(SYS_LANDLOCK_CREATE_RULESET), long(0), long(0), long(LANDLOCK_CREATE_RULESET_VERSION))
  if abi < 0 then
    M.containment.reason = 'Landlock indisponível: ' .. errstr(e)
    return
  end
  M.containment.abi = abi
  if ffi.C.prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) ~= 0 then
    M.containment.reason = 'PR_SET_NO_NEW_PRIVS: ' .. errstr(ffi.errno())
    return
  end
  local attr = ffi.new('kinein_landlock_attr[1]')
  attr[0].handled_access_fs = LANDLOCK_ACCESS_FS_EXECUTE
  local fd
  fd, e = syscall(long(SYS_LANDLOCK_CREATE_RULESET), ffi.cast('void *', attr), long(ffi.sizeof('kinein_landlock_attr')), long(0))
  if fd < 0 then
    M.containment.reason = 'landlock_create_ruleset: ' .. errstr(e)
    return
  end
  -- TSYNC aplica o domínio a todas as threads do processo, não só à que chama.
  local r
  r, e = syscall(long(SYS_LANDLOCK_RESTRICT_SELF), long(fd), long(LANDLOCK_RESTRICT_SELF_TSYNC))
  M.containment.tsync = r == 0
  if r ~= 0 and e == EINVAL then
    r, e = syscall(long(SYS_LANDLOCK_RESTRICT_SELF), long(fd), long(0))
  end
  ffi.C.close(fd)
  if r ~= 0 then
    M.containment.reason = 'landlock_restrict_self: ' .. errstr(e)
    return
  end
  M.containment.layer2 = true
end

-- A sonda que o core usa antes de entregar o editor: cada tentativa precisa
-- falhar. Se uma executar, o core mata o nvim (falha fechado).
function M.exec_probe()
  local result = {}
  local function spawn(name, file, args)
    local handle, err = vim.uv.spawn(file, { args = args }, function() end)
    if handle then
      result[name] = 'executou'
      handle:close()
    else
      result[name] = tostring(err)
    end
  end
  spawn('true', '/bin/true', {})
  spawn('ldso', '/lib64/ld-linux-x86-64.so.2', { '/bin/true' })
  spawn('nvim', vim.v.progpath, { '--version' })
  local ok, out = pcall(vim.fn.system, { '/bin/true' })
  result.system = ok and ('v:shell_error=' .. vim.v.shell_error) or tostring(out)
  result.containment = M.containment
  return result
end

local function core()
  if M._core then
    return M._core
  end
  for _, c in ipairs(vim.api.nvim_list_chans()) do
    if c.stream == 'stdio' and c.mode == 'rpc' then
      M._core = c.id
      return c.id
    end
  end
end

local function ui()
  local chan = vim.g.kinein_ui_chan
  if chan and chan > 0 then
    return chan
  end
end

local function notify_core(method, ...)
  local chan = core()
  if chan then
    vim.rpcnotify(chan, method, ...)
  end
end

local function set_options()
  local o = vim.o
  -- O projeto é texto: nada dele configura o editor.
  o.modeline = false
  o.modelineexpr = false
  o.exrc = false
  o.secure = true
  o.undofile = false
  o.swapfile = vim.env.KINEIN_PROVA_RECUPERACAO == 'swap'
  if vim.env.KINEIN_PROVA_ESTADO then
    o.directory = vim.env.KINEIN_PROVA_ESTADO .. '/swap//'
    vim.fn.mkdir(vim.env.KINEIN_PROVA_ESTADO .. '/swap', 'p')
  end
  o.updatetime = 300
  o.termguicolors = true
  o.number = true
  o.signcolumn = 'yes'
  o.mouse = 'a'
  -- Sem 'unnamedplus': com ele, todo apagamento (digitar ou colar sobre a
  -- seleção) troca a área de transferência da pessoa pelo texto apagado.
  -- Só Ctrl+C e Ctrl+X escrevem no "+, como no JetBrains.
  o.clipboard = ''
  o.showmode = false
  o.shortmess = o.shortmess .. 'AIcF'
  o.expandtab = true
  o.shiftwidth = 4
  o.tabstop = 4
  o.completeopt = 'menuone,noinsert'
  o.confirm = false
  o.hidden = true
  if not M.modal then
    -- Shift com as setas seleciona; digitar substitui a seleção.
    o.keymodel = 'startsel,stopsel'
    o.selectmode = 'mouse,key'
    o.selection = 'exclusive'
    o.whichwrap = 'b,s,<,>,[,]'
  end
end

local function disable_runtime_plugins()
  local g = vim.g
  for _, name in ipairs({
    'loaded_python3_provider', 'loaded_ruby_provider', 'loaded_perl_provider', 'loaded_node_provider',
  }) do
    g[name] = 0
  end
  -- Os plugins do runtime que executam programa: o Landlock já os impede, e
  -- desligá-los evita a mensagem de erro no meio da edição.
  for _, name in ipairs({
    'loaded_netrw', 'loaded_netrwPlugin', 'loaded_gzip', 'loaded_tarPlugin', 'loaded_tar',
    'loaded_zipPlugin', 'loaded_zip', 'loaded_remote_plugins', 'loaded_spellfile_plugin',
    'loaded_man', 'loaded_tutor_mode_plugin', 'loaded_2html_plugin',
  }) do
    g[name] = 1
  end
end

local function set_clipboard()
  local memory = { ['+'] = { { '' }, 'v' }, ['*'] = { { '' }, 'v' } }
  local function copy(reg)
    return function(lines, regtype)
      memory[reg] = { lines, regtype }
      local chan = ui()
      if chan then
        pcall(vim.rpcrequest, chan, 'kinein_clipboard_set', lines, regtype)
      end
    end
  end
  local function paste(reg)
    return function()
      local chan = ui()
      if chan then
        local ok, res = pcall(vim.rpcrequest, chan, 'kinein_clipboard_get')
        if ok and type(res) == 'table' then
          return res
        end
      end
      return memory[reg]
    end
  end
  vim.g.clipboard = {
    name = 'kinein',
    copy = { ['+'] = copy('+'), ['*'] = copy('*') },
    paste = { ['+'] = paste('+'), ['*'] = paste('*') },
    cache_enabled = 0,
  }
end

-- A posição em UTF-16, a unidade do QString e da acessibilidade do Qt.
local function cursor_utf16()
  local row, col = unpack(vim.api.nvim_win_get_cursor(0))
  local line = vim.api.nvim_get_current_line()
  local ok, idx = pcall(vim.str_utfindex, line, 'utf-16', math.min(col, #line), false)
  return row, ok and idx or col, line
end

function M.set_cursor_char(col16)
  local row = vim.api.nvim_win_get_cursor(0)[1]
  local line = vim.api.nvim_get_current_line()
  local ok, byte = pcall(vim.str_byteindex, line, 'utf-16', col16, false)
  vim.api.nvim_win_set_cursor(0, { row, ok and byte or col16 })
end

local function notify_cursor()
  local chan = ui()
  if chan then
    local row, col, line = cursor_utf16()
    vim.rpcnotify(chan, 'kinein_cursor', row, col, line)
  end
end

-- A única porta pela qual a Kinein muda texto: com o changedtick esperado.
-- Um changedtick velho é recusado, e quem pediu consulta de novo (R6).
function M.apply(buf, expected_tick, first, last, lines)
  if buf == 0 then
    buf = vim.api.nvim_get_current_buf()
  end
  local current = vim.api.nvim_buf_get_changedtick(buf)
  if current ~= expected_tick then
    return { ok = false, changedtick = current }
  end
  vim.api.nvim_buf_set_lines(buf, first, last, true, lines)
  return { ok = true, changedtick = vim.api.nvim_buf_get_changedtick(buf) }
end

-- Abre o arquivo; com `lines`, põe o texto do rascunho por cima e deixa o
-- buffer modificado: a recuperação não grava por conta própria.
function M.open(path, lines)
  vim.cmd('silent keepalt edit ' .. vim.fn.fnameescape(path))
  local buf = vim.api.nvim_get_current_buf()
  if lines then
    vim.api.nvim_buf_set_lines(buf, 0, -1, true, lines)
  end
  if not M.modal then
    vim.cmd.startinsert()
  end
  return { buf = buf, changedtick = vim.api.nvim_buf_get_changedtick(buf) }
end

function M.recover(path)
  vim.cmd('silent! recover! ' .. vim.fn.fnameescape(path))
  local buf = vim.api.nvim_get_current_buf()
  if not M.modal then
    vim.cmd.startinsert()
  end
  return { buf = buf, changedtick = vim.api.nvim_buf_get_changedtick(buf), name = vim.api.nvim_buf_get_name(buf) }
end

function M.buffer_text(path)
  for _, buf in ipairs(vim.api.nvim_list_bufs()) do
    if vim.api.nvim_buf_is_loaded(buf) and vim.api.nvim_buf_get_name(buf) == path then
      return {
        lines = vim.api.nvim_buf_get_lines(buf, 0, -1, true),
        changedtick = vim.api.nvim_buf_get_changedtick(buf),
        modified = vim.bo[buf].modified,
      }
    end
  end
  return vim.NIL
end

function M.state()
  local row, col16 = cursor_utf16()
  local mode = vim.api.nvim_get_mode().mode
  local clients = {}
  for _, c in ipairs(vim.lsp.get_clients()) do
    table.insert(clients, { id = c.id, name = c.name, root_dir = c.root_dir, buffers = vim.tbl_keys(c.attached_buffers) })
  end
  return {
    mode = mode,
    buf = vim.api.nvim_get_current_buf(),
    name = vim.api.nvim_buf_get_name(0),
    row = row,
    col16 = col16,
    changedtick = vim.api.nvim_buf_get_changedtick(0),
    modified = vim.bo.modified,
    clients = clients,
    diagnostics = #vim.diagnostic.get(0),
    swapfile = vim.bo.swapfile,
  }
end

-- O LSP: o core lança o servidor e devolve o socket do relé. O Neovim só
-- conecta; não cria processo nem escolhe argumento.
function M.lsp_start(buf)
  local chan = core()
  if not chan then
    return
  end
  local path = vim.api.nvim_buf_get_name(buf)
  local ok, cfg = pcall(vim.rpcrequest, chan, 'kinein_lsp_config', vim.bo[buf].filetype, path)
  if not ok or type(cfg) ~= 'table' then
    return
  end
  vim.lsp.start({
    name = cfg.name,
    cmd = vim.lsp.rpc.connect(cfg.socket),
    root_dir = cfg.root_dir,
    settings = type(cfg.settings) == 'table' and cfg.settings or nil,
    init_options = type(cfg.init_options) == 'table' and cfg.init_options or nil,
  }, { bufnr = buf })
end

-- O contexto mudou no core (outro preset, outra toolchain): o cliente velho
-- sai, e cada buffer da linguagem conecta de novo com a configuração nova.
function M.lsp_restart(name)
  local buffers = {}
  for _, client in ipairs(vim.lsp.get_clients({ name = name })) do
    for b in pairs(client.attached_buffers) do
      buffers[b] = true
    end
    client:stop(true)
  end
  vim.defer_fn(function()
    for b in pairs(buffers) do
      if vim.api.nvim_buf_is_valid(b) then
        M.lsp_start(b)
      end
    end
  end, 200)
  return vim.tbl_count(buffers)
end

local function comment_line()
  local row = vim.api.nvim_win_get_cursor(0)[1]
  local ok = pcall(function()
    require('vim._comment').toggle_lines(row, row, { row, 0 })
  end)
  if not ok then
    vim.cmd('normal gcc')
  end
end

function M.duplicate_line()
  local row, col = unpack(vim.api.nvim_win_get_cursor(0))
  vim.api.nvim_buf_set_lines(0, row, row, true, { vim.api.nvim_get_current_line() })
  vim.api.nvim_win_set_cursor(0, { row + 1, col })
end

function M.move_line(delta)
  local row, col = unpack(vim.api.nvim_win_get_cursor(0))
  local target = row + delta
  if target < 1 or target > vim.api.nvim_buf_line_count(0) then
    return
  end
  local first = math.min(row, target)
  local pair = vim.api.nvim_buf_get_lines(0, first - 1, first + 1, true)
  vim.api.nvim_buf_set_lines(0, first - 1, first + 1, true, { pair[2], pair[1] })
  vim.api.nvim_win_set_cursor(0, { target, col })
end

local function copy_line(cut)
  vim.fn.setreg('+', vim.api.nvim_get_current_line() .. '\n', 'l')
  if cut then
    vim.cmd('delete _')
  end
end

-- A tabela do 65 §12.3, sem modo: o buffer vive na inserção, a seleção é o
-- modo de seleção do Vim (digitar substitui) e cada atalho é um <Cmd> que não
-- tira a pessoa da inserção.
local function set_keymaps()
  local function map(modes, lhs, rhs, opts)
    vim.keymap.set(modes, lhs, rhs, vim.tbl_extend('force', { silent = true }, opts or {}))
  end
  map({ 'i', 's' }, '<C-s>', '<Cmd>write<CR>')
  map('i', '<C-z>', '<Cmd>undo<CR>')
  map('i', '<C-S-z>', '<Cmd>redo<CR>')
  map('s', '<C-z>', '<C-g><Esc><Cmd>undo<CR>')
  map('s', '<C-S-z>', '<C-g><Esc><Cmd>redo<CR>')
  -- Cada palavra é um passo do desfazer, como no JetBrains.
  map('i', '<Space>', '<C-g>u<Space>')
  map('i', '<CR>', function()
    return vim.fn.pumvisible() == 1 and '<C-y>' or '<C-g>u<CR>'
  end, { expr = true })
  map('i', '<Esc>', function()
    return vim.fn.pumvisible() == 1 and '<C-e>' or '<Cmd>nohlsearch<CR>'
  end, { expr = true })
  map({ 'i', 's' }, '<C-a>', '<Esc>gg0vG$<C-g>')
  map('s', '<C-c>', '<C-g>"+ygv<C-g>')
  map('s', '<C-x>', '<C-g>"+d')
  map('s', '<BS>', '<C-g>"_d')
  map('s', '<Del>', '<C-g>"_d')
  -- Com um snippet ativo (o completion aceito), Tab e Shift+Tab saltam entre
  -- os campos, como o padrão do Neovim que estes mapeamentos substituem.
  local function snippet_or(direction, keys)
    return function()
      if vim.snippet.active({ direction = direction }) then
        return string.format('<Cmd>lua vim.snippet.jump(%d)<CR>', direction)
      end
      return keys
    end
  end
  map('s', '<Tab>', snippet_or(1, '<C-g>>gv<C-g>'), { expr = true })
  map('s', '<S-Tab>', snippet_or(-1, '<C-g><gv<C-g>'), { expr = true })
  map('i', '<C-c>', function() copy_line(false) end)
  map('i', '<C-x>', function() copy_line(true) end)
  map('i', '<C-d>', M.duplicate_line)
  map('i', '<C-y>', '<Cmd>delete _<CR>')
  map('i', '<C-/>', comment_line)
  map('i', '<M-S-Up>', function() M.move_line(-1) end)
  map('i', '<M-S-Down>', function() M.move_line(1) end)
  map('i', '<S-Tab>', snippet_or(-1, '<C-d>'), { expr = true })
  map('i', '<C-Space>', function() vim.lsp.completion.get() end)
  map('i', '<C-b>', function() vim.lsp.buf.definition() end)
  map('i', '<S-F6>', function() vim.lsp.buf.rename() end)
  map('i', '<M-CR>', function() vim.lsp.buf.code_action() end)
  map('i', '<C-f>', '<C-o>/')
end

local function set_autocmds()
  local group = vim.api.nvim_create_augroup('kinein', { clear = true })
  local au = function(events, opts)
    vim.api.nvim_create_autocmd(events, vim.tbl_extend('force', { group = group }, opts))
  end
  au({ 'BufReadPost', 'BufNewFile' }, {
    callback = function(a)
      notify_core('kinein_buf_open', a.buf, vim.api.nvim_buf_get_name(a.buf))
    end,
  })
  au('BufWritePost', {
    callback = function(a)
      notify_core('kinein_buf_written', a.buf, vim.api.nvim_buf_get_name(a.buf), vim.api.nvim_buf_get_changedtick(a.buf))
    end,
  })
  if vim.env.KINEIN_PROVA_GRAVACAO == 'core' then
    -- O core grava, com a mesma guarda de hoje: o texto vai com o changedtick.
    au('BufWriteCmd', {
      callback = function(a)
        local res = vim.rpcrequest(core(), 'kinein_write', a.buf, vim.api.nvim_buf_get_name(a.buf),
          vim.api.nvim_buf_get_lines(a.buf, 0, -1, true), vim.bo[a.buf].eol, vim.api.nvim_buf_get_changedtick(a.buf))
        if type(res) == 'table' and res.ok then
          vim.bo[a.buf].modified = false
        else
          vim.notify('Kinein: gravação recusada: ' .. vim.inspect(res), vim.log.levels.ERROR)
        end
      end,
    })
  end
  au({ 'CursorMoved', 'CursorMovedI', 'TextChanged', 'TextChangedI', 'BufEnter' }, { callback = notify_cursor })
  au('FileType', {
    pattern = { 'c', 'cpp', 'rust' },
    callback = function(a)
      M.lsp_start(a.buf)
    end,
  })
  au('LspAttach', {
    callback = function(a)
      local client = vim.lsp.get_client_by_id(a.data.client_id)
      if client and client:supports_method('textDocument/completion') then
        vim.lsp.completion.enable(true, client.id, a.buf, { autotrigger = true })
      end
    end,
  })
  if not M.modal then
    -- A emulação do 'insertmode' (vim_diff.txt): voltar ao normal devolve a
    -- pessoa à inserção. Quem quer o modo do Vim liga a opção.
    au('ModeChanged', {
      pattern = '*:n',
      callback = function()
        vim.schedule(function()
          if vim.api.nvim_get_mode().mode == 'n' and vim.bo.buftype == '' then
            vim.cmd.startinsert()
          end
        end)
      end,
    })
    au('BufWinEnter', { callback = function() vim.cmd.startinsert() end })
  end
  au('VimEnter', {
    callback = function()
      notify_core('kinein_ready', M.containment)
    end,
  })
end

-- Colar sobre a seleção: o vim.paste do Neovim apaga a seleção com
-- `normal! <Del>`, e a seleção aberta na inserção volta como `niI` (normal à
-- espera de um comando, como depois de um i_CTRL-O); a tecla seguinte viraria
-- comando do modo normal. Aqui, depois do vim.paste do Neovim, o cursor vai
-- para depois do texto colado e um <Cmd><CR> vazio é o comando que o `niI`
-- espera: a inserção recomeça ali. Só a colagem de uma vez (fase -1, a da UI).
-- O vim.paste é o ponto de extensão documentado.
local function set_paste()
  local builtin = vim.paste
  vim.paste = function(lines, phase)
    local before = vim.api.nvim_get_mode().mode
    local ok = builtin(lines, phase)
    if phase == -1 and before:find('^[sS\19]') and vim.api.nvim_get_mode().mode == 'niI' then
      local row, col = unpack(vim.api.nvim_win_get_cursor(0))
      local line = vim.api.nvim_get_current_line()
      if col < #line then
        col = col + vim.str_utf_end(line, col + 1) + 1
      end
      vim.api.nvim_win_set_cursor(0, { row, col })
      vim.api.nvim_feedkeys(vim.keycode('<Cmd><CR>'), 'n', false)
    end
    return ok
  end
end

-- O vim.snippet seleciona o campo pondo o cursor no último caractere, o que
-- supõe 'selection' inclusive; com a exclusive (a do Shift+setas), o último
-- caractere fica fora, e digitar sobre `value` deixa `7e`. A seleção do campo
-- é feita com a inclusive, e a exclusive volta ao sair da seleção (o fim do
-- menu de completion, `ic:i`, vem antes das teclas do snippet e não conta).
-- Sem campo a selecionar (o último, ou vazio), ela volta pelo <Cmd> posto na
-- fila depois das teclas do snippet; um vim.schedule rodaria antes delas
-- quando o expand vem da resposta do servidor (o caso do rust-analyzer).
function M._snippet_selected()
  if not vim.api.nvim_get_mode().mode:find('^[sS\19]') then
    vim.o.selection = 'exclusive'
  end
end

local function set_snippet_selection()
  local function inclusive_while_selecting(fn)
    return function(...)
      vim.o.selection = 'inclusive'
      local result = fn(...)
      vim.api.nvim_feedkeys(vim.keycode("<Cmd>lua require('kinein')._snippet_selected()<CR>"), 'n', false)
      return result
    end
  end
  vim.snippet.expand = inclusive_while_selecting(vim.snippet.expand)
  vim.snippet.jump = inclusive_while_selecting(vim.snippet.jump)
  vim.api.nvim_create_autocmd('ModeChanged', {
    group = vim.api.nvim_create_augroup('kinein_snippet', { clear = true }),
    pattern = '[sS\19]:*',
    callback = function()
      vim.o.selection = 'exclusive'
    end,
  })
end

function M.setup()
  disable_runtime_plugins()
  set_options()
  set_clipboard()
  if not M.modal then
    set_paste()
    set_snippet_selection()
  end
  vim.ui.open = function(target)
    notify_core('kinein_ui_open', target)
    return nil, nil
  end
  vim.diagnostic.config({ virtual_text = true, signs = true, update_in_insert = true })
  if not M.modal then
    set_keymaps()
  end
  set_autocmds()
end

return M
