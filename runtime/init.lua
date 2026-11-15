-- O init.lua da Kinein, passado por `-u`. Nada da configuração da pessoa nem
-- do projeto entra: o runtimepath tem só o runtime da Kinein e o do Neovim, e
-- os caminhos de módulo Lua e C não olham para o diretório de trabalho.
local here = vim.fs.dirname(debug.getinfo(1, 'S').source:sub(2))

vim.o.runtimepath = here .. ',' .. vim.env.VIMRUNTIME
vim.o.packpath = ''
-- O padrão do LuaJIT procura `./?.lua` e `./?.so`: o diretório de trabalho é
-- o projeto, e um `require` de nome ausente carregaria código dele.
package.path = here .. '/lua/?.lua;' .. here .. '/lua/?/init.lua'
package.cpath = ''

local kinein = require('kinein')
-- Primeiro a contenção: nada abaixo roda antes de o processo perder o execve.
kinein.contain()
kinein.setup()
