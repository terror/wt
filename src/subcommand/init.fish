function wt
  switch "$argv[1]"
    case create c remove r switch s
      for arg in $argv
        switch "$arg"
          case -h --help
            command wt $argv
            return $status
          case --
            break
        end
      end

      command wt $argv | read --local --null --delimiter '' dir
      set -l exit_code $pipestatus[1]

      string match --quiet --regex '(?s)\A(?<dir>.*?)(?:\n)?\z' -- "$dir"

      if test -n "$dir"
        builtin cd "$dir"; or return $status
      end

      return $exit_code
    case '*'
      command wt $argv
  end
end
