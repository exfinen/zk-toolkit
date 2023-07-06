let a1 = Fq1 3
let b1 = Fq1 5
let c1 = Fq1 7
a1 * b1
b1 * c1

let a2 = Fq2 a1 b1
let b2 = Fq2 b1 c1
a2 * b2
-- testing inv a2*b2
inv a2 * b2

