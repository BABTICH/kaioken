export interface TerminalLine {
  type: "prompt" | "agent" | "wrong" | "correct" | "code" | "system" | "tool"
  text: string
  pause?: number
}

export const WITHOUT_KAIOKEN_SCRIPT: TerminalLine[] = [
  { type: "prompt", text: "> Add Redis rate-limiting to api/users" },
  { type: "wrong", text: "✖ Searching randomly in 84 files without module graph..." },
  { type: "agent", text: "Editing internal/server/main.go directly on main branch..." },
  { type: "wrong", text: "✖ Overwrote config without checking environment schema" },
  { type: "code", text: "+ import \"github.com/go-redis/redis/v8\" // deprecated package" },
  { type: "wrong", text: "✖ Build failed: compilation error in main.go:89" },
  { type: "prompt", text: "> fix the build error, you used the wrong package!" },
  { type: "wrong", text: "✖ Hallucinating different imports... 14 tests failing" },
  { type: "wrong", text: "✖ Your git working tree is now dirty with broken changes" },
  { type: "wrong", text: "[ INTERRUPTED ] Developer gives up and fixes manually" },
]

export const WITH_KAIOKEN_SCRIPT: TerminalLine[] = [
  { type: "prompt", text: "> Add Redis rate-limiting to api/users" },
  { type: "system", text: "● read_knowledge modules/cache & modules/api" },
  { type: "tool", text: "● checking repo skills: .agent/skills/redis-client" },
  { type: "system", text: "● git worktree add .kaioken/worktrees/agent-rate-limit" },
  { type: "tool", text: "● applying change using project-standard redis v9 client" },
  { type: "code", text: "+ rateLimiter := redis.NewLimiter(rdb, 100*time.Second)" },
  { type: "system", text: "● running autonomous verification gate: go test ./api/users/..." },
  { type: "correct", text: "✓ all 28 tests passed · 0 compiler warnings · zero lock contention" },
  { type: "correct", text: "✓ isolated git worktree verified · ready for your review" },
  { type: "prompt", text: "apply edit to main repo? [y]es [n]o > y" },
  { type: "correct", text: "✓ applied clean commit · flow state preserved" },
]
