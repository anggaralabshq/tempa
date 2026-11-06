# Renders docs/demo/demo.gif: `vhs docs/demo/demo.tape` from the repo root (needs vhs and tempa on PATH).
Output docs/demo/demo.gif
Set Shell "bash"
Set FontSize 15
Set Width 1000
Set Height 560
Set Padding 18
Set Theme "Catppuccin Mocha"
Set TypingSpeed 45ms

Hide
Type "source docs/demo/setup.sh && clear" Enter
Sleep 500ms
Show

Type "# A repo set up for Claude Code only" Enter Sleep 600ms
Type "ls -A" Enter Sleep 1.8s
Type "tempa import --yes | tail -4" Enter Sleep 2.5s
Type "clear && tempa doctor | head -7" Enter Sleep 4.5s
Type "clear && echo 'Also check the migration.' >> .claude/commands/review.md" Enter Sleep 600ms
Type "tempa sync --check" Enter Sleep 2.5s
Type "tempa import --yes | grep tempa/commands; tempa sync --check" Enter Sleep 3s
Type "tail -2 .tempa/commands/review.md .gemini/commands/review.toml" Enter Sleep 4s
