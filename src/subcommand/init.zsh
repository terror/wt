wt() {
  case "$1" in
    convert|cv|create|c|remove|r|switch|s)
      local dir

      dir=$(command wt "$@") || return $?

      if [ -n "$dir" ]; then
        builtin cd "$dir" || return $?

        local hook

        hook=$(command wt hook post-worktree-change 2>/dev/null)

        if [ -n "$hook" ]; then
          eval "$hook"
        fi
      fi

      ;;
    *)
      command wt "$@"
      ;;
  esac
}
