#!/usr/bin/env bash
set -euo pipefail
if [[ "${GITHUB_ACTIONS:-}" != true || "${RUNNER_ENVIRONMENT:-}" != github-hosted || "${RUNNER_OS:-}" != Linux || -z "${RUNNER_TEMP:-}" ]]; then
  echo 'Native fixture policy setup is restricted to disposable GitHub-hosted Linux runners' >&2
  exit 1
fi
# Permit only the owned test executable to create WebKit sandbox namespaces.
# The global Ubuntu restriction and the WebKit sandbox remain enabled.
profile_path="${RUNNER_TEMP}/aegis-native-userns.profile"
cat > "$profile_path" <<'PROFILE'
profile aegis-native-ci /tmp/aegis-native-*/NativeFixture/aegis-core flags=(unconfined) {
  userns,
}
PROFILE
sudo apparmor_parser -r "$profile_path"
if [[ -r /proc/sys/kernel/apparmor_restrict_unprivileged_userns ]]; then
  echo "Global namespace restriction remains $(cat /proc/sys/kernel/apparmor_restrict_unprivileged_userns)"
fi
