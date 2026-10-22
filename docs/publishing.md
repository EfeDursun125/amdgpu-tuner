# Publishing this repository to GitHub

The source tree includes its GPLv3 license, locked Cargo dependencies, vendored Material components, SVG icon, documentation screenshot and CI configuration. Build outputs and local test captures are ignored by Git.

1. Create an **empty** GitHub repository named `amdgpu-tuner` with your preferred visibility. The license and README already exist here, so leave GitHub's initialization options unchecked.
2. From this project directory, review and commit the source:

   ```sh
   git add .
   git diff --cached --stat
   git commit -m "Prepare AMDGPU Tuner 0.1.1"
   git branch -M main
   ```

3. Run the `git remote add origin` command shown on **your new repository's page**, then upload:

   ```sh
   git push -u origin main
   ```

4. Open the repository's **Actions** tab and review the first CI run. Local checks do not prove a hosted runner has completed successfully.
5. Confirm that the README icon and screenshot render on GitHub and that GitHub recognizes the GPL-3.0 license.

The alternative source ZIP is generated with `python3 scripts/package-source.py`. Its checksum is in `dist/SHA256SUMS`; generated archives are intentionally excluded from the source history.

For a GitHub Release, use the **AppImage release** Actions workflow via a matching `v*` tag, a deliberate `appimage` branch push, or its manual Run workflow button. It builds and tests on Ubuntu 24.04, then publishes the AppImage, matching source-with-dependencies ZIP, build-info and `SHA256SUMS`. The exact glibc requirement is included in the release description. See [AppImage packaging](appimage.md) for local packaging; `THIRD_PARTY.md` records bundled-component notices and provenance.
