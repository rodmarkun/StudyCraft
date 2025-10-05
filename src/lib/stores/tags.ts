import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export type Tag = string;

function createTagsStore() {
    const { subscribe, set, update } = writable<Tag[]>([]);

    return {
        subscribe,
        loadTags: async () => {
            try {
                const tags = await invoke<string[]>('get_available_tags');
                console.log('Loaded tags:', tags);
                set(tags);
            } catch (error) {
                console.error('Failed to load tags:', error);
                set([]);
            }
        },
        addTag: async (tag: string) => {
            try {
                console.log('Adding tag:', tag);
                await invoke('add_available_tag', { tag });
                update(tags => [...tags, tag]);
                console.log('Tag added successfully');
                return true;
            } catch (error) {
                console.error('Failed to add tag:', error);
                throw error;
            }
        },
        removeTag: async (tag: string) => {
            try {
                await invoke('remove_available_tag', { tag });
                update(tags => tags.filter(t => t !== tag));
                return true;
            } catch (error) {
                console.error('Failed to remove tag:', error);
                throw error;
            }
        }
    };
}

export const tagsStore = createTagsStore();