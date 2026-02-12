# Release Workflow

Follow this checklist every time you update the application, add features, or fix bugs. This ensures consistency and proper version control.

## 1. Prepare Release

### 1.1 Update Version Numbers

Update the version number in the following files. Use [Semantic Versioning](https://semver.org/) (e.g., `1.0.0` -> `1.0.1` for patches, `1.1.0` for features).

- **`package.json`**: Update `"version": "x.x.x"`
- **`src-tauri/tauri.conf.json`**: Update `"version": "x.x.x"`

### 1.2 Update Documentation

- **`README.md`**: If new features were added or installation steps changed, update the relevant sections.
- **`srs.md`**: If core requirements or database schemas changed, update the System Requirements Specification.

### 1.3 Update Changelog

- **`CHANGELOG.md`**: Add a new entry for the version.
  - Header format: `## [x.x.x] - YYYY-MM-DD`
  - Categories: `### Added`, `### Changed`, `### Fixed`, `### Removed`
  - List all significant changes since the last release.

### 1.4 Update Calculations

- **`CALCULATIONS.md`**: Update the calculations if any changes were made.
  - follow previous structure.

## 2. Validation

### 2.1 Run Tests

Ensure the application runs correctly in development mode.

```bash
npm run tauri dev
```

- Check console for errors.
- Verify new features work as expected.

### 2.2 Verify Build Configuration

Ensure `src-tauri/tauri.conf.json` has the correct `productName` and `identifier` if they were changed.

## 3. Build Application

Generate the production executable and installer.

```bash
npm run tauri build
```

- **Output Location**: `src-tauri/target/release/bundle/nsis/` (for Windows installer `.exe`)
- Test the generated `.exe` on a clean environment if possible to ensure offline functionality works.

## 4. Version Control (Git)

### 4.1 Commit Changes

Stage and commit all changes, including the version bumps and changelog.

```bash
git add .
git commit -m "chore: release version x.x.x"
```

### 4.2 Create Tag

Tag the commit with the version number.

```bash
git tag vx.x.x
```

_Example: `git tag v0.2.0`_

### 4.3 Push to Remote

Push the commit and the tags to the remote repository (GitHub/GitLab).

```bash
git push origin main
git push origin --tags
```

## 5. Deployment (Optional)

- Upload the `.exe` installer from `src-tauri/target/release/bundle/nsis/` to your release page (e.g., GitHub Releases).
- Notify users of the update.
