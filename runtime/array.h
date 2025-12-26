typedef struct {
    unsigned int num_elems;
    unsigned int capacity;
    unsigned int elem_size;
    unsigned int ref_count;
    void* elems;
} QLArray;

QLArray* __ql__QLArray_new(void* elems, unsigned int num_elems, unsigned long elem_size);
void __ql__QLArray_add_ref(QLArray* array);
void __ql__QLArray_remove_ref(QLArray* array);

void* __ql__QLArray_index(QLArray* array, unsigned int index);
