# Function blocks and displays

Ready-made blocks are under **Shift+A → Function blocks**. Inputs are on the left, outputs on the right, and bits are ordered from the least significant at the top (D0, Q0…). The number of bits or select inputs is set in Properties.

| Block | Symbol | Function |
| --- | --- | --- |
| Multiplexer | MUX | select S routes one of the D inputs to Y (1–3 select inputs) |
| Decoder, demultiplexer | X/Y | output Y with the number on A is 1, others 0; EN = 0 clears all. With data on EN it works as a demultiplexer |
| Priority encoder | HPRI | A shows the number of the highest active input I; V = 1 when any input is active (4:2 or 8:3) |
| Adder | Σ | adds A, B and carry CI; sum S and carry CO (the 1-bit one is a full adder), 1–4 bits |
| Comparator | COMP | compares A and B: A<B, A=B, A>B; 1–4 bits |
| BCD to 7-segment decoder | BCD/7SEG | turns 0–15 on D into segments a–g (10–15 as A–F) |
| Counter | CTR | on a rising edge of C counts up (+) or down (−); R clears at once, EN = 0 stops counting; CO = 1 at the highest state (0 when counting down); 2–8 bits |
| Register | RG | stores D on a rising edge of C, R clears |
| Shift register | SRG | on a rising edge of C shifts by one bit (Q0 → Q1…) and loads SI into Q0 |

> **Tip:** R and EN may stay unconnected: unconnected R means “no reset”, unconnected EN means “enabled”. Other unconnected inputs read as unknown X.

## Displays

- The **seven-segment display** has inputs a–g and the decimal point dp. A segment is lit at 1.
- The **hex display** shows 0–F from the 4-bit input D0–D3. Unknown input shows dark grey segments.
- In Properties you can choose the side of the pins (left, right, top, bottom). The digit stays upright, unlike rotating with R.

> **Tip:** The example Decade counter with a seven-segment display resets a counter with an AND gate at 10 (1010) – this is how a counter to any number is made.
