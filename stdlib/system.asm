; echo
  MKFN FN_cabbage_arbitrary
  FSIG
  WRFN
; dissasemble
  MKFN FN_noodle_bananas
  FSIG
  WRFN
; __draw_cool_box
  MKFN FN_kangaroo_cabbage
  FSIG
  WRFN
; help
  MKFN FN_noodle_dolphin
  FSIG
  WRFN
; fn cabbage_arbitrary
; __syscalld(PRNT, s)
; s
  REF VAR_noodle_pointbreak
  PRNT
  RET
; fn noodle_bananas
; nil
  PUSH false
  RET
; fn kangaroo_cabbage
; {
;     // Grab the length of the longest line/title
;     max_len = max(
;         (for l in lines do len(l)) + len(title)
;     )
; 
;     out = []
; 
;     // Header portion
;     out += format("╔{}╗", ['═'.repeat(max_len+2)])
;     out += format("║ {} ║", [title.pad_right(max_len)])
;     out += format("╠{}╣", ['═'.repeat(max_len+2)])
; 
;     // Body portion
;     for line in lines {
;         out += format("║ {} ║", [line.pad_right(max_len)])
;     }
; 
;     // Footer portion
;     out += format("╚{}╝", ['═'.repeat(max_len+2)])
; 
;     out.join('\n')
; }
  SCI
; max_len = max(
;         (for l in lines do len(l)) + len(title)
;     )
; max(
;         (for l in lines do len(l)) + len(title)
;     )
; for l in lines do len(l)) + len(title)
; for l in lines do len(l)
  MKAR 00000000
  REF VAR_jellybean_alabaster
  SCI
  DUP
  JMPNE JUMP_bananas_arbitrary
JUMP_bananas_cabbage:
  JMP JUMP_cabbage_bananas
  NEXT
  REF VAR_quarantine_umbrella
  WREF
  POP
JUMP_bananas_arbitrary:
  SWP
  REF VAR_quarantine_umbrella
  CALL FC1400FACF92C78 1
  PSAR
  SWP
  SCO
  JMP JUMP_bananas_cabbage
  SCO
  POP
  REF VAR_tangerine_grapefruit
  CALL FC1400FACF92C78 1
JUMP_cabbage_bananas:
  ADD
  CALL 8960430CFD85939F 1
  REF VAR_grapefruit_hedgehog
  WREF
  POP
  MKAR 00000000
  REF VAR_marmalade_jellybean
  WREF
  POP
  DUP
  PUSH `╔{}╗`
  PUSH `═`
  REF VAR_grapefruit_hedgehog
  PUSH 2
  ADD
  CALL 76D90C956114F2C2 2
  MKAR 00000001
  CALL D9468344D3651243 2
  LCST
  ADD
  REF VAR_marmalade_jellybean
  WREF
  POP
  DUP
  PUSH `║ {} ║`
  REF VAR_tangerine_grapefruit
  REF VAR_grapefruit_hedgehog
  CALL D38975B17B432A76 2
  MKAR 00000001
  CALL D9468344D3651243 2
  LCST
  ADD
  REF VAR_marmalade_jellybean
  WREF
  POP
  DUP
  PUSH `╠{}╣`
  PUSH `═`
  REF VAR_grapefruit_hedgehog
  PUSH 2
  ADD
  CALL 76D90C956114F2C2 2
  MKAR 00000001
  CALL D9468344D3651243 2
  LCST
  ADD
  REF VAR_marmalade_jellybean
  WREF
  POP
  MKAR 00000000
  REF VAR_jellybean_alabaster
  SCI
  DUP
  JMPNE JUMP_rhubarb_dolphin
  JMP JUMP_salamander_pointbreak
  NEXT
  REF VAR_salamander_kangaroo
  WREF
  POP
  SWP
  SCI
  DUP
  PUSH `║ {} ║`
JUMP_noodle_alabaster:
  REF VAR_salamander_kangaroo
JUMP_rhubarb_dolphin:
  REF VAR_grapefruit_hedgehog
  CALL D38975B17B432A76 2
  MKAR 00000001
  CALL D9468344D3651243 2
  LCST
  ADD
  REF VAR_marmalade_jellybean
  WREF
  SCO
  PSAR
  SWP
  SCO
  JMP JUMP_noodle_alabaster
  SCO
  POP
  POP
  DUP
  PUSH `╚{}╝`
  PUSH `═`
  REF VAR_grapefruit_hedgehog
  PUSH 2
JUMP_salamander_pointbreak:
  ADD
  CALL 76D90C956114F2C2 2
  MKAR 00000001
  CALL D9468344D3651243 2
  LCST
  ADD
  REF VAR_marmalade_jellybean
  WREF
  POP
  REF VAR_marmalade_jellybean
  PUSH `
`
  CALL F7E2C8231C57A8BD 2
  SCO
  RET
; fn noodle_dolphin
; {
;     functions = __syscalld(LSTFN)
; 
;     // Now we reorganize the functions into a dictionary by category
;     // We also filter out any functions that start with '__'
;     categories = {}
;     for f in functions {
;         f = f as object
;         __syscalld(PRNT, "Processing function: " + f['name'])
; 
;         if !(categories contains f['category']) {
;             categories[f['category']] = []
;         } else nil
;         categories[f['category']] = f
;     } where !((f as object)['name'] starts_with '__')
; 
;     __syscalld(PRNT, "Got categories: " + categories)
; 
;     // If we're not filtering, we just list all the functions
;     if filter == "" {
;         out = ""
;         for category in categories {
;             lines = []
;             for f in categories[category] {
;                 line = f['signature']
;                 if f contains 'short' then line += ' ' + f['short'] else nil
;                 lines += line
;             }
; 
;             out += __draw_cool_box(category, lines) + '\n'
;             return out
;         }
;     } else {
;         // If we are filtering, we only show the functions that match
;         out = ""
;         for category in categories {
;             functions = for f in categories[category] {
;                 title = f['signature']
;                 lines = []
;                 if f contains 'short' then lines += f['short'] else nil
;                 if f contains 'desc' {
;                     for l in __syscalld(SSPLT, f['desc'], '\n') {
;                         lines += l
;                     }
;                 } else nil
; 
;                 if f contains 'example' {
;                     for l in __syscalld(SSPLT, f['example'], '\n') {
;                         lines += l
;                     }
;                 } else nil
; 
;                 out += __draw_cool_box(title, lines) + '\n'
;             } where {
;                 f['name'] contains filter || f['category'] contains filter
;             }
;         }
;         
;         return out
;     }
; }
  SCI
JUMP_salamander_cabbage:
; functions = __syscalld(LSTFN)
; __syscalld(LSTFN)
  LSTFN
; functions
  REF VAR_salamander_lumberjack
  WREF
  POP
; categories = {}
; {}
  MKOB 00000000
  REF VAR_bananas_marmalade
  WREF
  POP
  MKAR 00000000
  REF VAR_salamander_lumberjack
  SCI
  DUP
  JMPNE JUMP_alabaster_arbitrary
  JMP JUMP_hedgehog_bananas
JUMP_grapefruit_alabaster:
  NEXT
  REF VAR_umbrella_noodle
  WREF
  REF VAR_umbrella_noodle
JUMP_alabaster_arbitrary:
  CAST Object
  PUSH `name`
  IDEX
  PUSH `__`
  STWT
  LNOT
  JMPT JUMP_salamander_cabbage
  JMP JUMP_hedgehog_bananas
  SWP
  SCI
  REF VAR_umbrella_noodle
  CAST Object
  REF VAR_umbrella_noodle
  WREF
  POP
  PUSH `Processing function: `
  REF VAR_umbrella_noodle
  PUSH `name`
  IDEX
  ADD
  PRNT
  POP
  REF VAR_bananas_marmalade
  REF VAR_umbrella_noodle
  PUSH `category`
  IDEX
  CNTN
  LNOT
  JMPF JUMP_umbrella_dolphin
  SCI
  MKAR 00000000
  REF VAR_bananas_marmalade
  REF VAR_umbrella_noodle
  PUSH `category`
  IDEX
  IDEX
  WREF
  SCO
  JMP JUMP_dolphin_pointbreak
  PUSH false
  POP
  REF VAR_umbrella_noodle
  REF VAR_bananas_marmalade
  REF VAR_umbrella_noodle
  PUSH `category`
JUMP_dolphin_pointbreak:
JUMP_umbrella_dolphin:
  IDEX
  IDEX
  WREF
  SCO
  PSAR
  SWP
  SCO
  JMP JUMP_grapefruit_alabaster
  SCO
  POP
  POP
  PUSH `Got categories: `
  REF VAR_bananas_marmalade
  ADD
  PRNT
JUMP_hedgehog_bananas:
  POP
  REF VAR_tangerine_octopus
  PUSH ``
  EQ
  JMPF JUMP_quarantine_umbrella
  SCI
  PUSH ``
  REF VAR_marmalade_jellybean
  WREF
  POP
  MKAR 00000000
  REF VAR_bananas_marmalade
  SCI
  DUP
  JMPNE JUMP_penguin_grapefruit
  JMP JUMP_arbitrary_hedgehog
  NEXT
  REF VAR_arbitrary_penguin
  WREF
  POP
  SWP
  SCI
  MKAR 00000000
  REF VAR_jellybean_alabaster
  WREF
  POP
  MKAR 00000000
  REF VAR_bananas_marmalade
JUMP_bananas_octopus:
  REF VAR_arbitrary_penguin
  IDEX
  SCI
  DUP
JUMP_penguin_grapefruit:
  JMPNE JUMP_alabaster_jellybean
  JMP JUMP_marmalade_kangaroo
  NEXT
  REF VAR_umbrella_noodle
  WREF
  POP
  SWP
  SCI
  REF VAR_umbrella_noodle
  PUSH `signature`
  IDEX
  REF VAR_salamander_kangaroo
JUMP_quarantine_noodle:
  WREF
  POP
  REF VAR_umbrella_noodle
  PUSH `short`
JUMP_alabaster_jellybean:
  CNTN
  JMPF JUMP_kangaroo_lumberjack
  DUP
  PUSH ` `
  REF VAR_umbrella_noodle
  PUSH `short`
  IDEX
  ADD
  LCST
  ADD
  REF VAR_salamander_kangaroo
  WREF
  JMP JUMP_salamander_marmalade
  PUSH false
  POP
  DUP
  REF VAR_salamander_kangaroo
  LCST
  ADD
  REF VAR_jellybean_alabaster
  WREF
  SCO
  PSAR
  SWP
  SCO
  JMP JUMP_quarantine_noodle
  SCO
  POP
  POP
  DUP
  REF VAR_arbitrary_penguin
  REF VAR_jellybean_alabaster
  CALL 9B64F27B7E116322 2
  PUSH `
`
JUMP_salamander_marmalade:
JUMP_kangaroo_lumberjack:
  ADD
  LCST
  ADD
  REF VAR_marmalade_jellybean
  WREF
  POP
  REF VAR_marmalade_jellybean
  RET
  SCO
  PSAR
  SWP
  SCO
  JMP JUMP_bananas_octopus
  SCO
JUMP_marmalade_kangaroo:
  POP
  SCO
  JMP JUMP_tangerine_penguin
  SCI
  PUSH ``
  REF VAR_marmalade_jellybean
  WREF
  POP
  MKAR 00000000
  REF VAR_bananas_marmalade
  SCI
  DUP
  JMPNE JUMP_cabbage_quarantine
  JMP JUMP_penguin_rhubarb
  NEXT
  REF VAR_arbitrary_penguin
  WREF
  POP
  SWP
  SCI
  MKAR 00000000
  REF VAR_bananas_marmalade
  REF VAR_arbitrary_penguin
JUMP_arbitrary_hedgehog:
  IDEX
  SCI
  DUP
  JMPNE JUMP_rhubarb_salamander
JUMP_quarantine_umbrella:
  JMP JUMP_umbrella_tangerine
  NEXT
  REF VAR_umbrella_noodle
  WREF
  SCI
  REF VAR_umbrella_noodle
  PUSH `name`
JUMP_rhubarb_arbitrary_penguin:
  IDEX
  REF VAR_tangerine_octopus
  CNTN
  REF VAR_umbrella_noodle
JUMP_cabbage_quarantine:
  PUSH `category`
  IDEX
  REF VAR_tangerine_octopus
  CNTN
  LOR
  SCO
  JMPT JUMP_salamander_cabbage
  JMP JUMP_umbrella_tangerine
JUMP_penguin_arbitrary_octopus:
  SWP
  SCI
  REF VAR_umbrella_noodle
  PUSH `signature`
JUMP_rhubarb_salamander:
  IDEX
  REF VAR_tangerine_grapefruit
  WREF
  POP
  MKAR 00000000
  REF VAR_jellybean_alabaster
  WREF
  POP
  REF VAR_umbrella_noodle
  PUSH `short`
  CNTN
  JMPF JUMP_jellybean_arbitrary_cabbage
  DUP
  REF VAR_umbrella_noodle
  PUSH `short`
  IDEX
  LCST
  ADD
  REF VAR_jellybean_alabaster
  WREF
  JMP JUMP_penguin_arbitrary_dolphin
  PUSH false
  POP
  REF VAR_umbrella_noodle
  PUSH `desc`
  CNTN
  JMPF JUMP_cabbage_arbitrary_pointbreak
  SCI
  MKAR 00000000
  PUSH `
`
  REF VAR_umbrella_noodle
  PUSH `desc`
  IDEX
  SSPLT
  SCI
  DUP
  JMPNE JUMP_lumberjack_arbitrary_alabaster
  JMP JUMP_umbrella_arbitrary_umbrella
  NEXT
  REF VAR_quarantine_umbrella
  WREF
  POP
  SWP
  SCI
  DUP
  REF VAR_quarantine_umbrella
  LCST
  ADD
  REF VAR_jellybean_alabaster
  WREF
  SCO
  PSAR
  SWP
  SCO
  JMP JUMP_rhubarb_arbitrary_grapefruit
  SCO
  POP
  SCO
  JMP JUMP_cabbage_arbitrary_hedgehog
JUMP_jellybean_arbitrary_cabbage:
JUMP_penguin_arbitrary_dolphin:
  PUSH false
  POP
  REF VAR_umbrella_noodle
  PUSH `example`
  CNTN
  JMPF JUMP_tangerine_arbitrary_jellybean
  SCI
  MKAR 00000000
  PUSH `
`
  REF VAR_umbrella_noodle
  PUSH `example`
JUMP_rhubarb_arbitrary_grapefruit:
  IDEX
  SSPLT
  SCI
  DUP
  JMPNE JUMP_alabaster_arbitrary_kangaroo
  JMP JUMP_dolphin_arbitrary_lumberjack
  NEXT
  REF VAR_quarantine_umbrella
JUMP_lumberjack_arbitrary_alabaster:
  WREF
  POP
  SWP
  SCI
  DUP
  REF VAR_quarantine_umbrella
  LCST
  ADD
  REF VAR_jellybean_alabaster
  WREF
  SCO
  PSAR
  SWP
  SCO
  JMP JUMP_marmalade_arbitrary_marmalade
  SCO
  POP
  SCO
  JMP JUMP_cabbage_arbitrary_noodle
  PUSH false
JUMP_umbrella_arbitrary_umbrella:
  POP
  DUP
  REF VAR_tangerine_grapefruit
JUMP_cabbage_arbitrary_pointbreak:
  REF VAR_jellybean_alabaster
JUMP_cabbage_arbitrary_hedgehog:
  CALL 9B64F27B7E116322 2
  PUSH `
`
  ADD
  LCST
  ADD
  REF VAR_marmalade_jellybean
  WREF
  SCO
  PSAR
  SWP
  SCO
  JMP JUMP_penguin_arbitrary_octopus
  SCO
  POP
  REF VAR_salamander_lumberjack
  WREF
  SCO
  PSAR
  SWP
  SCO
  JMP JUMP_rhubarb_arbitrary_penguin
  SCO
  POP
  POP
  REF VAR_marmalade_jellybean
JUMP_marmalade_arbitrary_marmalade:
  RET
  SCO
  SCO
  RET
JUMP_penguin_rhubarb:
JUMP_dolphin_arbitrary_lumberjack:
JUMP_cabbage_arbitrary_noodle:
JUMP_tangerine_arbitrary_jellybean:
JUMP_alabaster_arbitrary_kangaroo:
JUMP_tangerine_penguin:
JUMP_umbrella_tangerine:
