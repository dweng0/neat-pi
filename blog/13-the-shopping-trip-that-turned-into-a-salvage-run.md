---
title: "The shopping trip that turned into a salvage run"
episode: 13
pubDate: 2026-08-06
sessionDate: 2026-08-06
status: published
teaser: "I set aside a session to buy the battery parts. One by one, nearly every line item argued its way off the list — a multimeter I already own, two plant pots, a dead PC in the garage — until the only thing left to buy was wire I probably don't need either."
heroPhoto: battery-bms-pcb-closeup.jpg
seeAlso: [reference/handoff, reference/brain-transplant]
---

I blocked out this session for the boring bit: buy the battery parts. The pack's already open, the cells (`Samsung INR18650-35E`, `4S2P`) tested alive at `14.23 V`, the plan settled — reuse the cells, hang a dumb BMS on the bot, feed it from the dock through a small charger. All that was left was to fill a basket. Six line items. An afternoon of clicking *buy*.

Instead, one by one, nearly every item talked me out of it.

The BMS went in easily enough — a `4S 16.8V 40A` board with a balancer built in. But that "40 A" nagged. My cells are only good for about `16 A` continuous (two `35E`s in parallel, `~8 A` each). A 40 A BMS won't trip until *long* after the cells are in trouble. It's not wrong — a dead short still pulls hundreds of amps and trips it instantly — but there's a middle band, a `20–35 A` partial fault, it'll happily ignore while the wiring cooks. So the BMS stays, and it earns a companion: a plain inline fuse, sized to the *pack*, not the board.

Which sent me to fuses, and the first small lesson. A glass cartridge fuse looked fine — `250 V` rating — until I remembered that's AC, and those glass tubes have feeble breaking capacity. A lithium pack shorted dead sprays more current than a glass fuse can cleanly snuff; it can arc across the tube instead of clearing. The right part is the boring one every car already carries: a `20 A` blade fuse, DC-rated, in a `£3` inline holder. Ordered. And then I talked myself into *deferring even that* — the BMS covers dead shorts on its own, I'm watching every bench session, and the fuse only truly earns its keep once the robot's roaming unwatched. On the list, off the critical path.

Then the voltage sensor, and the near-miss I'm gladdest I caught. The obvious buy is a "`25 V` voltage sensor module" — a little divider board. Except it divides by 5, for a `5 V` Arduino. My pack tops out at `16.8 V`; divide that by 5 and you get `3.36 V`, and my ESP32's ADC ceiling is `3.3 V`. It would clip at exactly the full-charge end I most want to read. So — don't buy it. Two resistors from the Elegoo kit, `10k` up top and `2k` down, map `16.8 V` to a clean `2.8 V`, tuned to my pack instead of someone's `5 V` board. That's not a purchase. That's ten minutes with a soldering iron.

By now a pattern was forming, and the connector broke it wide open. Why buy an `XT60` disconnect for a robot that's *full* of connectors? The old battery harness carried full pack current in its former life — salvage it. Same story for the wire, and here I nearly slipped. I've got a coil of twin-and-earth in the garage, fat enough to carry `20 A` without blinking. But it's *solid core*, and this thing vibrates: brushes, wheels, a blower, hours of trundling. Solid copper work-hardens and cracks at the joints under that. Twin-and-earth is for walls that never move. What I want is *stranded* — and a dead PC's power supply is stuffed with it. Better still: if I leave the PSU's main cables intact and only harvest the spare SATA and Molex tails, that supply becomes the `12 V` bench source I need to spin the wheel motor next session anyway. Jump the green wire to black and it turns on. The best part on the whole list turned out to be a corpse in my garage.

Even the safety kit dissolved. The pack lives in a small terracotta pot, nested inside a bigger one, on the concrete garage floor. Fired clay doesn't burn. That's the fireproof box — bought years ago, for plants.

I ended the session where I started, staring at a shopping list, except three things were ordered and everything else had a line through it and a note beside it: *already own this.* Then I did the one thing that genuinely needed a screen — sketched the whole power spine in Mermaid, dock to cells to BMS to bus to motors — and it immediately started asking harder questions than any product page had. Where does the charger tie in: before the fuse, or after the disconnect? Where does every ground actually come together?

That's next session's fight. Right after the wheel finally turns.
