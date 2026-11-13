# Contributing

Thanks for your interest in Kuerdas Studio. Contributions that improve reliable guitar recording, backing tracks, arrangement, mixing, accessibility and documentation are welcome.

## Before you start

- Check the [roadmap](docs/roadmap.md) and current [limitations](README.md#project-status) so proposals fit the recording-studio scope.
- For a substantial feature or change to audio behavior, open an issue first to discuss the approach.
- Do not include recordings, workspace data, credentials, private device identifiers or unlicensed impulse responses in a change.

## Changes and checks

Keep audio callbacks free of blocking locks, allocation, file access and UI communication. Put disk and graph work on control or worker threads. Preserve existing workspace and recording data when changing persistence.

Before opening a pull request, run the checks in [Development](docs/development.md#validate-changes). Include a concise description, user-visible behavior, checks performed and any hardware that you tested. Hardware checks are helpful but must be distinguished from automated test results.

## Licensing

By submitting a contribution, you agree that it can be distributed under this repository's [MIT License](LICENSE). Keep notices for third-party code and assets, and confirm you have the rights to contribute them.
