import { create } from 'zustand';
import { kanbanApi, type LocalOrganization } from '@/shared/api/kanbanApi';

interface LocalOrganizationsState {
  organizations: LocalOrganization[];
  selectedOrgId: string | null;
  isLoading: boolean;
  error: string | null;
}

export const useLocalOrganizationStore = create<LocalOrganizationsState & {
  fetchOrganizations: () => Promise<void>;
  createOrganization: (name: string) => Promise<LocalOrganization>;
  setSelectedOrgId: (orgId: string | null) => void;
  getSelectedOrganization: () => LocalOrganization | null;
  clearAll: () => void;
}>((set, get) => ({
  organizations: [],
  selectedOrgId: null,
  isLoading: false,
  error: null,

  fetchOrganizations: async () => {
    set({ isLoading: true, error: null });
    try {
      const organizations = await kanbanApi.listOrganizations();
      set({ 
        organizations: organizations,
        isLoading: false 
      });
    } catch (err) {
      console.error('[useLocalOrganizationStore] Failed to fetch organizations:', err);
      set({ 
        error: err instanceof Error ? err.message : 'Failed to fetch organizations',
        isLoading: false 
      });
    }
  },

  createOrganization: async (name: string) => {
    set({ isLoading: true, error: null });
    try {
      const newOrg = await kanbanApi.createOrganization(name);
      set((state) => ({
        organizations: [...state.organizations, newOrg],
        selectedOrgId: newOrg.id,
        isLoading: false,
      }));
      return newOrg;
    } catch (err) {
      console.error('[useLocalOrganizationStore] Failed to create organization:', err);
      set({ 
        error: err instanceof Error ? err.message : 'Failed to create organization',
        isLoading: false 
      });
      throw err;
    }
  },

  setSelectedOrgId: (orgId: string | null) => {
    localStorage.setItem('vk-local-selected-org', orgId || '');
    set({ selectedOrgId: orgId });
  },

  getSelectedOrganization: () => {
    const state = get();
    return state.organizations.find((org) => org.id === state.selectedOrgId) || null;
  },

  clearAll: () => {
    set({ organizations: [], selectedOrgId: null });
    localStorage.removeItem('vk-local-selected-org');
  },
}));

const storedOrgId = localStorage.getItem('vk-local-selected-org');
if (storedOrgId) {
  useLocalOrganizationStore.getState().setSelectedOrgId(storedOrgId);
}
