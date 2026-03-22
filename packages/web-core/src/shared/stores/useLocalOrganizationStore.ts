import { create } from 'zustand';
import type { Organization } from 'shared/types';

const STORAGE_KEY = 'vk-local-organizations';

interface LocalOrganization extends Organization {
  isLocal: true;
}

interface LocalOrganizationsState {
  organizations: LocalOrganization[];
  selectedOrgId: string | null;
}

const loadFromStorage = (): LocalOrganizationsState => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored) as LocalOrganizationsState;
      return {
        organizations: parsed.organizations || [],
        selectedOrgId: parsed.selectedOrgId || null,
      };
    }
  } catch {
    // localStorage may be unavailable or corrupted
  }
  return { organizations: [], selectedOrgId: null };
};

const saveToStorage = (state: LocalOrganizationsState): void => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // localStorage may be unavailable
  }
};

export const useLocalOrganizationStore = create<LocalOrganizationsState & {
  createOrganization: (name: string, slug: string) => Organization;
  setSelectedOrgId: (orgId: string | null) => void;
  getSelectedOrganization: () => Organization | null;
  clearAll: () => void;
}>((set, get) => ({
  ...loadFromStorage(),

  createOrganization: (name: string, slug: string) => {
    const now = new Date().toISOString();
    const newOrg: LocalOrganization = {
      id: `local-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
      name,
      slug,
      is_personal: true,
      issue_prefix: 'LOCAL',
      created_at: now,
      updated_at: now,
      isLocal: true,
    };

    set((state) => {
      const newState = {
        ...state,
        organizations: [...state.organizations, newOrg],
        selectedOrgId: newOrg.id,
      };
      saveToStorage(newState);
      return newState;
    });

    return newOrg;
  },

  setSelectedOrgId: (orgId: string | null) => {
    set((state) => {
      const newState = { ...state, selectedOrgId: orgId };
      saveToStorage(newState);
      return newState;
    });
  },

  getSelectedOrganization: () => {
    const state = get();
    return state.organizations.find((org) => org.id === state.selectedOrgId) || null;
  },

  clearAll: () => {
    const newState = { organizations: [], selectedOrgId: null };
    saveToStorage(newState);
    set(newState);
  },
}));
