import QtQuick

// Resultado do git.status → GitController, e os momentos que a UI já
// conhece como "o status pode ter mudado" (save e fs-ops) → refresh.
Item {
    id: root

    property var coreClient: null
    property var gitController: null

    visible: false

    Connections {
        target: root.coreClient

        function onGitStatusResolved(repo, branch, detached, shortSha,
                                     ahead, behind, entries) {
            root.gitController.handleStatus(repo, branch, detached, shortSha,
                                            ahead, behind, entries);
        }

        function onGitBranchesResolved(repo, branches) {
            root.gitController.handleBranches(repo, branches);
        }

        function onGitRemoteOperationFinished(operation, success, message) {
            root.gitController.handleRemoteFinished(operation, success, message);
        }

        function onFileSaved(path) {
            root.gitController.refresh();
            if (path === root.gitController.activeDiffPath) {
                root.gitController.requestDiffFor(path);
            }
            if (root.gitController.blameVisible
                    && path === root.gitController.blamePath) {
                root.gitController.requestBlameFor(path);
            }
        }

        function onGitFileDiffResolved(path, repo, tracked, hunks, text) {
            root.gitController.handleFileDiff(path, repo, tracked, hunks,
                                              text);
        }

        function onGitBlameResolved(path, repo, tracked, groups) {
            root.gitController.handleBlame(path, repo, tracked, groups);
        }

        function onGitLogResolved(repo, entries) {
            root.gitController.handleLog(repo, entries);
        }

        function onGitCommitDiffResolved(sha, text) {
            root.gitController.handleCommitDiff(sha, text);
        }

        function onFileCreated(path) {
            root.gitController.refresh();
        }

        function onDirectoryCreated(path) {
            root.gitController.refresh();
        }

        function onPathRenamed(from, to) {
            root.gitController.refresh();
        }

        function onPathDeleted(path) {
            root.gitController.refresh();
        }

        function onFilesChanged(changes) {
            root.gitController.refresh();
            if (root.gitController.blameVisible) {
                for (let i = 0; i < changes.length; i++) {
                    if (changes[i].path === root.gitController.blamePath) {
                        root.gitController.requestBlameFor(changes[i].path);
                        break;
                    }
                }
            }
        }

        function onRequestFailed(method, message) {
            root.gitController.handleRequestFailed(method, message);
        }
    }
}
