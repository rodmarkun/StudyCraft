import { writable } from 'svelte/store';

export const focusedCollection = writable<string | null>(null);