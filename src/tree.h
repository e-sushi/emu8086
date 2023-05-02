typedef struct TNode {
    struct TNode* next;
    struct TNode* prev;
    struct TNode* parent;
    struct TNode* first_child;
    struct TNode* last_child;
    unsigned int child_count;

    char* debug;
} TNode;

void insert_after(struct TNode* target, struct TNode* node);
void insert_before(struct TNode* target, struct TNode* node);