#!/usr/bin/env bash
#
# Regenerate versions.json + the root redirect for the versioned site from the
# `vX.Y.Z/` snapshot directories present in a gh-pages working tree.
#
#   update-manifest.sh <gh-pages-dir>
#
# Writes:
#   <dir>/versions.json  {"latest": "<newest>", "versions": [<newest..oldest>]}
#   <dir>/index.html     redirect to /v<newest>/ (or /dev/ if no release yet)
#
# Pure/deterministic so it can be unit-tested without a deploy.
set -euo pipefail

GH="${1:?usage: update-manifest.sh <gh-pages-dir>}"

versions=()
for d in "$GH"/v*/; do
  [ -d "$d" ] || continue
  name="$(basename "$d")" # vX.Y.Z
  versions+=("${name#v}")  # X.Y.Z
done

# Sort newest-first by semver (numeric on each of the three fields).
sorted=()
if [ "${#versions[@]}" -gt 0 ]; then
  while IFS= read -r v; do sorted+=("$v"); done < <(
    printf '%s\n' "${versions[@]}" | sort -t. -k1,1nr -k2,2nr -k3,3nr
  )
fi

# Build the JSON array.
list=""
for i in "${!sorted[@]}"; do
  [ "$i" -gt 0 ] && list+=", "
  list+="\"${sorted[$i]}\""
done

if [ "${#sorted[@]}" -gt 0 ]; then
  latest="${sorted[0]}"
  redirect="/v${latest}/"
else
  latest=""
  redirect="/dev/"
fi

printf '{\n  "latest": "%s",\n  "versions": [%s]\n}\n' "$latest" "$list" > "$GH/versions.json"

cat > "$GH/index.html" <<HTML
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>playwright-rust</title>
<meta http-equiv="refresh" content="0; url=${redirect}">
<link rel="canonical" href="${redirect}">
</head>
<body>Redirecting to <a href="${redirect}">${redirect}</a>…</body>
</html>
HTML

echo "versions.json: latest=${latest:-<none>} versions=[${list}]"
echo "root redirect → ${redirect}"
