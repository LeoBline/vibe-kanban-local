import { create } from 'zustand';
import type { Project } from 'shared/remote-types';

const STORAGE_KEY = 'vk-local-projects';

interface LocalProject extends Project {
  isLocal: true;
}

interface LocalProjectsState {
  projects: LocalProject[];
}

const loadFromStorage = (): LocalProjectsState => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored) as LocalProjectsState;
      return {
        projects: parsed.projects || [],
      };
    }
  } catch {
    // localStorage may be unavailable or corrupted
  }
  return { projects: [] };
};

const saveToStorage = (state: LocalProjectsState): void => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // localStorage may be unavailable
  }
};

export const useLocalProjectStore = create<LocalProjectsState & {
  createProject: (organizationId: string, name: string, color: string) => Project;
  getProjectsByOrganization: (organizationId: string) => Project[];
  updateProject: (projectId: string, updates: Partial<Pick<Project, 'name' | 'color' | 'sort_order'>>) => void;
  deleteProject: (projectId: string) => void;
  clearAll: () => void;
}>((set, get) => ({
  ...loadFromStorage(),

  createProject: (organizationId: string, name: string, color: string) => {
    const now = new Date().toISOString();
    const state = get();
    const existingProjects = state.projects.filter(p => p.organization_id === organizationId);
    const maxSortOrder = existingProjects.length > 0 
      ? Math.max(...existingProjects.map(p => p.sort_order)) 
      : -1;

    const newProject: LocalProject = {
      id: `local-project-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
      organization_id: organizationId,
      name,
      color,
      sort_order: maxSortOrder + 1,
      created_at: now,
      updated_at: now,
      isLocal: true,
    };

    console.log('[DEBUG createProject] Before set', { 
      currentProjectsCount: state.projects.length,
      newProject 
    });

    set((state) => {
      const newState = {
        ...state,
        projects: [...state.projects, newProject],
      };
      console.log('[DEBUG createProject] After set', { 
        newProjectsCount: newState.projects.length,
        projectIds: newState.projects.map(p => p.id)
      });
      saveToStorage(newState);
      return newState;
    });

    return newProject;
  },

  getProjectsByOrganization: (organizationId: string) => {
    const state = get();
    return state.projects
      .filter((p) => p.organization_id === organizationId)
      .sort((a, b) => a.sort_order - b.sort_order);
  },

  updateProject: (projectId: string, updates: Partial<Pick<Project, 'name' | 'color' | 'sort_order'>>) => {
    set((state) => {
      const newState = {
        ...state,
        projects: state.projects.map((p) =>
          p.id === projectId
            ? { ...p, ...updates, updated_at: new Date().toISOString() }
            : p
        ),
      };
      saveToStorage(newState);
      return newState;
    });
  },

  deleteProject: (projectId: string) => {
    set((state) => {
      const newState = {
        ...state,
        projects: state.projects.filter((p) => p.id !== projectId),
      };
      saveToStorage(newState);
      return newState;
    });
  },

  clearAll: () => {
    const newState = { projects: [] };
    saveToStorage(newState);
    set(newState);
  },
}));
