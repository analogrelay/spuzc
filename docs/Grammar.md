# Grammar

```
FLOAT           := SIGN? DIGIT+ '.' DIGIT+ EXPONENT_PART?
INTEGER         := DECINT

DECINT          := SIGN? DIGIT+
SIGN            := '+' | '-'
EXPONENT_PART   := 'e' SIGN? DIGIT+ | 'E' SIGN? DIGIT+
DIGIT           := '0'..'9'
HEXDIGIT        := '0'..'9' | 'A'..'F' | 'a'..'f'
```