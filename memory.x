/* For NumWorks n0110 calculators */
MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x08000000, LENGTH = 64k
  RAM : ORIGIN = 0x20000000, LENGTH = 176K + 16K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
