#!/usr/bin/env python3
"""Mutações da fronteira entre documentação pública e orientações locais."""

from pathlib import Path
import os
import subprocess
import tempfile
import unittest

from check_public_docs_privacy import inspect_files, inspect_text


class PublicDocumentationPrivacyTests(unittest.TestCase):
    """Exemplos técnicos permanecem válidos; identificadores locais reprovam."""

    def test_generic_paths_and_runtime_sessions_are_technical(self) -> None:
        for text in (
            '/home/user/project', '/home/pi/project', '/home/<usuario>/project',
            'C:\\Users\\user\\project', '/Users/user/project',
            'Sessão de depuração: o prompt do terminal permanece disponível.',
            'Setas/Home/End/PgUp; MAC ilustrativo 02:00:00:00:00:01.',
            'Qt 6.12; GNU AGPLv3; atribuição pública e Signed-off-by.',
            'Preservar a sessão do agente no terminal integrado.',
        ):
            with self.subTest(text=text):
                self.assertEqual(inspect_text('guide.md', text), [])

    def test_personal_accounts_are_rejected_on_each_platform(self) -> None:
        for text in (
            '/home/local-account/project', '/Users/local-account/project',
            'C:\\Users\\local-account\\project', 'file:///home/local-account/main.cpp',
        ):
            with self.subTest(text=text):
                self.assertTrue(inspect_text('guide.md', text))

    def test_local_guides_and_session_metadata_are_rejected(self) -> None:
        for text in (
            '[guia](../CLAUDE.md)', 'AGENTS.md', 'DocsPrivate/state.md',
            '05-com-um-agente-de-ia.md', '**HANDOFF**', '**Checkout:** local',
            'Objetivo autorizado', 'Retomada local', 'prompt de entrada',
            'O autor instalou ferramentas', 'Modo autônomo', 'Meu notebook',
            'Renovar a cota da sessão', 'Modelo/esforço herdado',
            'Continuidade automática', 'Mais dois agentes adicionais',
            'O autor tornou LSP obrigatório e pediu ferramentas existentes',
            'Interrupção por limite semanal', 'A sessão do agente, que rodava no terminal',
            'Fornecer prompt para outra sessão', 'Agente passo 13',
        ):
            with self.subTest(text=text):
                self.assertTrue(inspect_text('guide.md', text))

    def test_device_identifiers_are_rejected(self) -> None:
        self.assertTrue(inspect_text('guide.md', 'MAC aa:bb:cc:dd:ee:ff'))

    def test_a_local_guide_cannot_be_selected_even_with_neutral_contents(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            for name in ('AGENTS.md', 'CLAUDE.md', 'nested/AGENTS.override.md',
                         'nested/05-com-um-agente-de-ia.md', 'DocsPrivate/notes.md',
                         '.claude/settings.json', '.codex/notes.md'):
                with self.subTest(name=name):
                    self.assertTrue(inspect_files(Path(directory), [name]))

    def test_git_ignores_local_guides_but_forced_tracking_is_rejected(self) -> None:
        git_env = {name: value for name, value in os.environ.items() if not name.startswith('GIT_')}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '--quiet', directory], env=git_env, check=True)
            (root / '.gitignore').write_text('CLAUDE.md\n')
            (root / 'CLAUDE.md').write_text('Orientações locais\n')
            command = ['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z']
            names = subprocess.check_output(command, cwd=root, env=git_env).decode().split('\0')
            self.assertNotIn('CLAUDE.md', names)
            subprocess.run(['git', 'add', '--force', 'CLAUDE.md'], cwd=root, env=git_env, check=True)
            names = subprocess.check_output(command, cwd=root, env=git_env).decode().split('\0')
            self.assertTrue(inspect_files(root, names))

    def test_mutating_a_clean_document_introduces_a_failure(self) -> None:
        base = 'Build: cargo build --locked\n'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / 'README.md'
            path.write_text(base)
            self.assertEqual(inspect_files(root, ['README.md']), [])
            for suffix in ('/home/local-account/build', 'CLAUDE.md', 'HANDOFF'):
                path.write_text(base + suffix)
                self.assertTrue(inspect_files(root, ['README.md']))


if __name__ == '__main__':
    unittest.main()
