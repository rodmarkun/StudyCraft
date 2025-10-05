import { writable } from 'svelte/store';
export type CoverImageCacheMap = Map<string, string>;

function createCoverImageCache() {
  const { subscribe, set, update } = writable<CoverImageCacheMap>(new Map());
  
  return {
    subscribe,
    set: (value: CoverImageCacheMap): void => set(value),
   
    get: (key: string): string | undefined => {
      let currentValue: CoverImageCacheMap | undefined;
      subscribe(val => {
        currentValue = val;
      })();
      return currentValue?.get(key);
    },
   
    has: (key: string): boolean => {
      let currentValue: CoverImageCacheMap | undefined;
      subscribe(val => {
        currentValue = val;
      })();
      return currentValue?.has(key) || false;
    },
   
    setItem: (key: string, value: string): void => {
      update(map => {
        map.set(key, value);
        return map;
      });
    },
   
    delete: (key: string): void => {
      update(map => {
        map.delete(key);
        return map;
      });
    },
   
    clear: (): void => {
      update(map => {
        map.forEach(url => {
          URL.revokeObjectURL(url);
        });
        return new Map<string, string>();
      });
    }
  };
}

export const coverImageCache = createCoverImageCache();

if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    coverImageCache.clear();
  });
}