import { StateStorage } from "zustand/middleware";
import { LazyStore } from "@tauri-apps/plugin-store";

export const preferencesStore = new LazyStore(".preferences.dat");

export const preferencesStorage: StateStorage = {
    getItem: async (name: string): Promise<string | null> => {
        return (await preferencesStore.get(name)) || null;
    },
    setItem: async (name: string, value: string): Promise<void> => {
        await preferencesStore.set(name, value);
    },
    removeItem: async (name: string): Promise<void> => {
        await preferencesStore.delete(name);
    },
};
