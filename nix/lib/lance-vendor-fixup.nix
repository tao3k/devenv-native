{ lanceSrc }:
''
  fix_lance_vendor_manifest_lints() {
    manifest="$1"
    tmp_manifest="''${manifest}.tmp"
    awk '
      $0 == "[lints]" {
        in_lints = 1
        next
      }
      in_lints && $0 == "workspace = true" {
        in_lints = 0
        next
      }
      {
        if (in_lints) {
          print "[lints]"
          in_lints = 0
        }
        print
      }
      END {
        if (in_lints) {
          print "[lints]"
        }
      }
    ' "$manifest" > "$tmp_manifest"
    mv "$tmp_manifest" "$manifest"
  }

  ensure_lance_arrow_ffi_dependency() {
    manifest="$1"
    dependency="$2"
    tmp_manifest="''${manifest}.tmp"
    awk -v dependency="$dependency" '
      function close_dependency_table() {
        if (in_dependency_table && !table_has_features) {
          print "features = [\"ffi\"]"
        }
        in_dependency_table = 0
        table_has_features = 0
      }
      function ensure_features_line_has_ffi() {
        compact = $0
        gsub(/[[:space:]]/, "", compact)
        if (compact ~ /features=\[\]/) {
          sub(/features[[:space:]]*=[[:space:]]*\[[^]]*\]/, "features = [\"ffi\"]")
        } else if (compact !~ /features=\[[^]]*"ffi"/) {
          sub(/features[[:space:]]*=[[:space:]]*\[/, "features = [\"ffi\", ")
        }
      }
      $0 ~ "^[[:space:]]*\\[dependencies\\." dependency "\\][[:space:]]*$" {
        close_dependency_table()
        in_dependency_table = 1
        table_has_features = 0
        print
        next
      }
      $0 ~ "^\\[" {
        close_dependency_table()
        print
        next
      }
      in_dependency_table && $0 ~ "^[[:space:]]*features[[:space:]]*=" {
        ensure_features_line_has_ffi()
        table_has_features = 1
        print
        next
      }
      $0 ~ "^[[:space:]]*" dependency "[[:space:]]*=" && $0 ~ /\{.*\}/ {
        close_dependency_table()
        compact = $0
        gsub(/[[:space:]]/, "", compact)
        if (compact !~ /features=\[/) {
          sub(/\}[[:space:]]*$/, ", features = [\"ffi\"] }")
        } else if (compact ~ /features=\[\]/) {
          sub(/features[[:space:]]*=[[:space:]]*\[[^]]*\]/, "features = [\"ffi\"]")
        } else if (compact !~ /features=\[[^]]*"ffi"/) {
          sub(/features[[:space:]]*=[[:space:]]*\[/, "features = [\"ffi\", ")
        }
      }
      { print }
      END {
        close_dependency_table()
      }
    ' "$manifest" > "$tmp_manifest"
    mv "$tmp_manifest" "$manifest"
  }

  materialize_lance_vendor_crate() {
    crate_dir="$1"
    if [ -L "$crate_dir" ]; then
      source_dir="$(realpath "$crate_dir")"
      tmp_dir="''${crate_dir}.tmp"
      rm -rf "$tmp_dir"
      mkdir -p "$tmp_dir"
      cp -a --no-preserve=mode,ownership "$source_dir"/. "$tmp_dir"/
      rm "$crate_dir"
      mv "$tmp_dir" "$crate_dir"
    fi
    chmod -R u+w "$crate_dir" 2>/dev/null || true
  }

  restore_lance_vendor_protos() {
    crate_dir="$1"
    [ -e "$crate_dir/protos" ] || [ -L "$crate_dir/protos" ] || return 0
    chmod -R u+w "$crate_dir/protos" 2>/dev/null || true
    rm -rf "$crate_dir/protos"
    cp -R ${lanceSrc}/protos "$crate_dir/protos"
    chmod -R u+w "$crate_dir/protos" 2>/dev/null || true
  }

  fix_lance_vendor_dir() {
    vendor_dir="$1"
    if [ ! -d "$vendor_dir" ]; then
      echo "missing cargo vendor directory: $vendor_dir" >&2
      exit 1
    fi

    for crate_dir in "$vendor_dir"/fsst-* "$vendor_dir"/lance-*; do
      [ -e "$crate_dir" ] || [ -L "$crate_dir" ] || continue
      materialize_lance_vendor_crate "$crate_dir"
      if [ -f "$crate_dir/Cargo.toml" ] \
        && grep -q '^\[lints\]$' "$crate_dir/Cargo.toml" \
        && grep -q '^workspace = true$' "$crate_dir/Cargo.toml"; then
        fix_lance_vendor_manifest_lints "$crate_dir/Cargo.toml"
      fi
      case "$(basename "$crate_dir")" in
        lance-io-*|lance-datafusion-*)
          ensure_lance_arrow_ffi_dependency "$crate_dir/Cargo.toml" arrow
          ensure_lance_arrow_ffi_dependency "$crate_dir/Cargo.toml" arrow-array
          ;;
      esac
      restore_lance_vendor_protos "$crate_dir"
    done
  }
''
