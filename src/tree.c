#include "tree.h"

void insert_after(struct TNode* target, struct TNode* node) {
    if(target->next) target->prev = node;
    node->next = target->next;
    node->prev = target;
    target->next = node;
}

void insert_before(struct TNode* target, struct TNode* node) {
    if (target->prev) target->prev->next = node;
	node->prev = target->prev;
	node->next = target;
	target->prev = node;
}