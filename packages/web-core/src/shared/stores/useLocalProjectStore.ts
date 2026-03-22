import { create } from 'zustand';
import type { Project } from 'shared/remote-types';
import { kanbanApi, type LocalProject } from '@/shared/api/kanbanApi';

interface LocalProjectsState {
  projects: LocalProject[];
  isLoading: boolean;
  error: string | null;
}

export const useLocalProjectStore = create<LocalProjectsState & {
  fetchProjects: (organizationId: string) => Promise<void>;
  createProject: (organizationId: string, name: string, color: string) => Promise<LocalProject>;
  getProjectsByOrganization: (organizationId: string) => Project[];
  updateProject: (projectId: string, updates: Partial<Pick<Project, 'name' | 'color' | 'sort_order'>>) => Promise<void>;
  deleteProject: (projectId: string) => Promise<void>;
  clearAll: () => void;
}>((set, get) => ({
  projects: [],
  isLoading: false,
  error: null,

  fetchProjects: async (organizationId: string) => {
    set({ isLoading: true, error: null });
    try {
      console.log('[useLocalProjectStore] fetchProjects called for org:', organizationId);
      const projects = await kanbanApi.listProjects(organizationId);
      console.log('[useLocalProjectStore] Received projects from API:', projects.length, projects.map(p => ({ id: p.id, name: p.name })));
      set({ 
        projects: projects,
        isLoading: false 
      });
      console.log('[useLocalProjectStore] Store updated with projects:', projects.length);
    } catch (err) {
      console.error('[useLocalProjectStore] Failed to fetch projects:', err);
      set({ 
        error: err instanceof Error ? err.message : 'Failed to fetch projects',
        isLoading: false 
      });
    }
  },

  createProject: async (organizationId: string, name: string, color: string) => {
    set({ isLoading: true, error: null });
    try {
      const newProject = await kanbanApi.createProject(organizationId, name, color);
      set((state) => ({
        projects: [...state.projects, newProject],
        isLoading: false,
      }));
      return newProject;
    } catch (err) {
      console.error('[useLocalProjectStore] Failed to create project:', err);
      set({ 
        error: err instanceof Error ? err.message : 'Failed to create project',
        isLoading: false 
      });
      throw err;
    }
  },

  getProjectsByOrganization: (organizationId: string) => {
    const state = get();
    return state.projects
      .filter((p) => p.organization_id === organizationId)
      .sort((a, b) => a.sort_order - b.sort_order);
  },

  updateProject: async (projectId: string, updates: Partial<Pick<Project, 'name' | 'color' | 'sort_order'>>) => {
    set({ isLoading: true, error: null });
    try {
      const updatedProject = await kanbanApi.updateProject(projectId, updates);
      set((state) => ({
        projects: state.projects.map((p) =>
          p.id === projectId ? updatedProject : p
        ),
        isLoading: false,
      }));
    } catch (err) {
      console.error('[useLocalProjectStore] Failed to update project:', err);
      set({ 
        error: err instanceof Error ? err.message : 'Failed to update project',
        isLoading: false 
      });
      throw err;
    }
  },

  deleteProject: async (projectId: string) => {
    set({ isLoading: true, error: null });
    try {
      await kanbanApi.deleteProject(projectId);
      set((state) => ({
        projects: state.projects.filter((p) => p.id !== projectId),
        isLoading: false,
      }));
    } catch (err) {
      console.error('[useLocalProjectStore] Failed to delete project:', err);
      set({ 
        error: err instanceof Error ? err.message : 'Failed to delete project',
        isLoading: false 
      });
      throw err;
    }
  },

  clearAll: () => {
    set({ projects: [], isLoading: false, error: null });
  },
}));
