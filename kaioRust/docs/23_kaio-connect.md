Zed gives the editor away and charges for hosted AI. You can copy that model, and your open-core plan (open CLI plus a hosted gateway) already fits it.

**How Zed earns**

| Plan | Price | What you get |
|---|---|---|
| Personal | Free | 2,000 accepted edit predictions, plus unlimited use with your own API keys or external agents like Claude Code |
| Pro | $10/month | Unlimited edit predictions, $5 of tokens included, then usage-based billing |
| Business | $30/seat/month | Organization AI controls, data governance, unified billing and role-based access |
| Enterprise | Contact sales | Custom security and support |

The money mostly comes from billing hosted usage at API list price plus 10%. They also run a two-week Pro trial with $20 of token credits.

**How kaioken can earn**

| Stream | How it works | When |
|---|---|---|
| Hosted gateway (your `kaioken connect`) | Users skip API keys and pay for tokens at cost plus 10-20% | 🥇 First |
| Pro subscription | $10-15/month with a small credit bundle | 🥇 First |
| Team plan | Shared memory, team dashboard, per-seat pricing | 🥈 After the first users |
| Enterprise | Self-hosted gateway, SSO, audit export (your `audit show` feature sells this) | 🥉 Later |
| Marketplace | Paid plugins and templates, you take a cut | Later |
| Sponsors / GitHub Sponsors | Donations for the open-source side | Anytime, easy |

**Suggested pricing**

| Plan | Price | Includes |
|---|---|---|
| Free | $0 | Full CLI, bring your own keys, unlimited |
| Pro | $12/month | Hosted gateway, $5 in credits, overage at cost +15%, cloud memory sync |
| Team | $25/seat/month | Pro plus shared memory, audit log, central billing |
| Enterprise | Custom | Self-hosted, SSO, support |

**Watch-outs**

| Risk | What to do |
|---|---|
| Heavy users cost more than they pay | Pass tokens through at cost plus a markup. Your Redis quota system already helps cap this |
| Paid features must be hosted | Keep anything that runs on your servers (gateway, sync, dashboard) closed. Keep the CLI open |
| Payments from Morocco | As far as I know, Stripe doesn't support Moroccan businesses directly. Check merchant-of-record services like Paddle or Lemon Squeezy, or a US entity setup, and verify before you build billing |
| Free users with no upgrade reason | Make the hosted gateway and cloud sync clearly easier than bring-your-own-keys |

Want me to write the billing spec (plans, quota logic, `kaioken connect` flow) in the same format as your docs 11-20?