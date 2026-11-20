#!/usr/bin/env bash
# Pořídí snímky obrazovky pro nápovědu: každý obvod z docs/napoveda/obvody
# otevře v programu ve virtuální obrazovce a uloží snímek celého okna
# do target/snimky/<název>.png (1600 × 1000 px).
#
# Potřebuje Linux s balíčky xvfb, xdotool a imagemagick:
#   sudo apt install xvfb xdotool imagemagick x11-utils
# Použití (z kořene repozitáře):
#   cargo build --release
#   scripts/snimky-napovedy.sh            # všechny obvody
#   scripts/snimky-napovedy.sh vodice     # jen jeden
#
# Výřez a popisky se dělají ručně; hotové obrázky patří do
# docs/napoveda/<jazyk>/src/obrazky/. Jazyk rozhraní určuje LANG
# (výchozí cs_CZ.UTF-8, pro anglickou nápovědu LANG=en_GB.UTF-8).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/logicflow"
OUT="$ROOT/target/snimky"
DISPLAY_NUM=":99"
LANG_UI="${LANG_UI:-${LANG:-cs_CZ.UTF-8}}"

[ -x "$BIN" ] || { echo "Chybí $BIN – nejdřív spusťte cargo build --release" >&2; exit 1; }
mkdir -p "$OUT"

if ! xdpyinfo -display "$DISPLAY_NUM" >/dev/null 2>&1; then
  Xvfb "$DISPLAY_NUM" -screen 0 1600x1000x24 >/dev/null 2>&1 &
  XVFB_PID=$!
  trap 'kill $XVFB_PID 2>/dev/null || true' EXIT
  sleep 1
fi
export DISPLAY="$DISPLAY_NUM"

if [ $# -gt 0 ]; then
  FILES=()
  for n in "$@"; do FILES+=("$ROOT/docs/napoveda/obvody/$n.logicflow"); done
else
  FILES=("$ROOT"/docs/napoveda/obvody/*.logicflow)
fi

for f in "${FILES[@]}"; do
  name="$(basename "$f" .logicflow)"
  home="$(mktemp -d)"   # čisté nastavení: výchozí rozložení a evropské značky
  HOME="$home" LANG="$LANG_UI" LIBGL_ALWAYS_SOFTWARE=1 "$BIN" "$f" >/dev/null 2>&1 &
  pid=$!
  sleep 4
  wid="$(xdotool search --name LogicFlow | head -1)"
  xdotool windowsize "$wid" 1600 1000 windowmove "$wid" 0 0
  sleep 1
  import -window root "$OUT/$name.png"
  kill "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  rm -rf "$home"
  echo "$OUT/$name.png"
done
