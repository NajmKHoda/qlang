#ifndef RUNTIME_PRIMITIVES_H
#define RUNTIME_PRIMITIVES_H

typedef struct QLTypeInfo QLTypeInfo;

extern QLTypeInfo __ql__int_type_info;
extern QLTypeInfo __ql__bool_type_info;

void printi(int);
void printb(bool);
int inputi();

#endif