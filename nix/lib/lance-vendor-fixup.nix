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

  fix_lance_vendor_dir() {
    vendor_dir="$1"
    if [ ! -d "$vendor_dir" ]; then
      echo "missing cargo vendor directory: $vendor_dir" >&2
      exit 1
    fi

    for crate_dir in "$vendor_dir"/fsst-* "$vendor_dir"/lance-*; do
      [ -e "$crate_dir" ] || continue
      materialize_lance_vendor_crate "$crate_dir"
      if [ -f "$crate_dir/Cargo.toml" ] \
        && grep -q '^\[lints\]$' "$crate_dir/Cargo.toml" \
        && grep -q '^workspace = true$' "$crate_dir/Cargo.toml"; then
        fix_lance_vendor_manifest_lints "$crate_dir/Cargo.toml"
      fi
    done

    for crate_name in \
      lance \
      lance-datafusion \
      lance-encoding \
      lance-file \
      lance-index \
      lance-table
    do
      for crate_dir in "$vendor_dir"/"''${crate_name}"-*; do
        [ -e "$crate_dir" ] || continue
        materialize_lance_vendor_crate "$crate_dir"
        rm -rf "$crate_dir/protos"
        cp -R ${lanceSrc}/protos "$crate_dir/protos"
      done
    done
  }
''
