# The Lighthouse of New Haven: solo campaign

The lighthouse keeps New Haven’s memories. At midnight it goes dark, and an apprentice archivist receives a compass carrying Lyra’s voice. Bringing the light back takes more than finding three keys: the apprentice must discover why the lighthouse went dark and who should control the city’s memory.

## How to play

1. On the Flask home page, select **«Маяк Новой Гавани · Играть одному»** (*The Lighthouse of New Haven · Play solo*). Guests can see the map; an account is required to play and save progress.
2. Choose an unlocked chapter, read its briefing and start the trial. No friends, room or countdown are needed. Read the feedback after each answer and select **Continue**.
3. Accuracy of at least 40%, 65% and 90% earns one, two and three stars respectively. One star reveals the chapter epilogue. Later chapters unlock based on total best stars, which can improve on replays.

| Act | Chapters | Story beat |
| --- | --- | --- |
| I. The Dark Archive | Midnight Signal → Hall of Timelines → Glass Conservatory → Archive Guardian | The guardian gives you the Memory Key: someone shut the lighthouse down on purpose. |
| II. Map of the Living City | Silent Cinema → Relay in the Square → Map on Water → Radio Workshop → Keeper of Routes | Lyra shut it down when it began erasing memories. The Link Key opens the tower. |
| III. Heart of the Lighthouse | Hall of Names → The Last Page → Dawn over New Haven | The Choice Key lets the city restore the light without giving up control of its memories. |

## Where things live

| Layer | Path | Responsibility |
| --- | --- | --- |
| Domain | `app/domain/campaign_catalog.py` | Acts, briefings, epilogues and chapter rules; no HTTP, database or AI calls. |
| Application service | `app/services/campaign.py` | Stars, progress and rewards backed by SQLite. |
| HTTP | `app/http_api/campaign_routes.py` | Sessions, access, question generation and JSON. |
| Presentation | `templates/partials/campaign.html`, `static/js/game.js` | Map, reading, answers and results; story text comes from the catalog. |

The existing level IDs 1–12, topics, thresholds and level settings are preserved; existing progress needs no migration. `GET /api/campaign/levels` returns the map, `POST /api/campaign/start` returns questions for an unlocked chapter, and `POST /api/campaign/result` records a result. Generation uses the configured AI or existing fallback bank. The narrative is fixed; questions can vary.

## Scope and verification

The campaign ships in the Flask application. Rust and SvelteKit do not get this feature automatically; that port needs its own API and data migration. The legacy result endpoint accepts client-reported totals, so it is not a trusted source for competitive rankings. That requires server-owned attempts, answer validation and replay protection. Refreshing the page does not restore an unfinished attempt.

`pytest -q tests/test_api.py` checks HTTP and chapter launch. `node --check static/js/game.js` checks client syntax. Manual path: guest → sign in → map → first chapter → answers and result → map with saved stars. See also the [system context](system-context.md).
