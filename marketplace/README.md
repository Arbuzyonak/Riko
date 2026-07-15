# Riko plugin marketplace

The launcher's **Plugins → Browse the marketplace** panel reads a single
`catalog.json` and installs plugins from it. There is no server: the catalog is
a static file (host it on GitHub Pages, a raw Git URL, or any static host) and
the plugin archives are plain `.zip` files (GitHub Releases work well).

The catalog URL defaults to `marketplace::DEFAULT_CATALOG_URL` and can be
overridden per install via `config.toml`:

```toml
[plugins]
catalog_url = "https://raw.githubusercontent.com/OWNER/riko-plugins/main/catalog.json"
```

## How installs are made safe

1. The client downloads the archive named by `download_url`.
2. It computes the SHA-256 of the bytes and **refuses to install** unless it
   matches the catalog's `sha256`. A swapped or corrupted archive is rejected.
3. The archive is extracted with path-traversal protection (no `..`, no
   absolute paths) into the user's plugin directory.
4. The extracted `plugin.toml` name must match the catalog `name`.
5. Only then is the normal build/install flow run — and the UI still warns that
   building runs the plugin's commands locally.

Checksums stop tampering in transit, but they do **not** make a plugin
trustworthy. Treat the catalog like a package registry: review submissions.

## Catalog format

`catalog.json` is `{ "plugins": [ ... ] }`. Each entry:

| field          | required | notes                                             |
|----------------|----------|---------------------------------------------------|
| `name`         | yes      | must equal the plugin folder / `plugin.toml` name |
| `version`      | yes      | display only                                      |
| `description`  | yes      | one line                                          |
| `kind`         | yes      | `vulkan-layer` \| `binary` \| `env-only`          |
| `platforms`    | no       | e.g. `["linux"]`; defaults to `["linux"]`         |
| `download_url` | yes      | direct link to the `.zip`                         |
| `sha256`       | yes      | hex digest of the `.zip`                          |
| `size_bytes`   | no       | shown in the UI                                   |
| `author`       | no       | shown in the UI                                   |
| `homepage`     | no       | project link                                      |

The `.zip` may contain the plugin files at its root or nested under a folder
named `name/` — both are accepted.

## Submitting a plugin (recommended flow)

1. Build your plugin folder (a `plugin.toml` plus its assets).
2. Run `./package-plugin.sh path/to/your-plugin` to produce the `.zip` and a
   catalog-entry stub with the size and checksum filled in.
3. Upload the `.zip` (a GitHub Release asset is easiest) and paste its link into
   the stub's `download_url`.
4. Open a PR adding the entry to `catalog.json`. Review + merge is the trust
   gate — CI can additionally validate the manifest and re-verify the checksum.
