import { useState, useEffect, useMemo, useCallback, type ReactNode } from 'react';
import { OrgContext, type OrgContextValue } from '@/shared/hooks/useOrgContext';
import type { InsertResult, MutationResult } from '@/shared/lib/electric/types';
import type { Project, CreateProjectRequest, UpdateProjectRequest } from 'shared/remote-types';
import { kanbanApi, type LocalProject } from '@/shared/api/kanbanApi';

interface LocalOrgProviderProps {
  organizationId: string;
  children: ReactNode;
}

export function LocalOrgProvider({ organizationId, children }: LocalOrgProviderProps) {
  const [projects, setProjects] = useState<LocalProject[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const loadProjects = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const data = await kanbanApi.listProjects(organizationId);
      setProjects(data);
    } catch (err) {
      console.error('Failed to load projects:', err);
      setError(err instanceof Error ? err : new Error('Failed to load projects'));
    } finally {
      setIsLoading(false);
    }
  }, [organizationId]);

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  const projectsById = useMemo(() => {
    const map = new Map<string, Project>();
    for (const project of projects) {
      const adaptedProject: Project = {
        id: project.id,
        name: project.name,
        color: project.color,
        organization_id: project.organization_id,
        sort_order: project.sort_order,
        created_at: project.created_at,
        updated_at: project.updated_at,
      };
      map.set(project.id, adaptedProject);
    }
    return map;
  }, [projects]);

  const getProject = (projectId: string) => projectsById.get(projectId);

  const insertProject = useCallback(
    (data: CreateProjectRequest): InsertResult<Project> => {
      const tempId = `temp-project-${Date.now()}`;
      const now = new Date().toISOString();
      const tempProject: Project = {
        id: tempId,
        name: data.name,
        color: data.color ?? '#3b82f6',
        organization_id: data.organization_id,
        sort_order: 0,
        created_at: now,
        updated_at: now,
      };

      setProjects(prev => [...prev, tempProject as unknown as LocalProject]);

      kanbanApi.createProject(data.organization_id, data.name, data.color).then(project => {
        const adaptedProject: Project = {
          id: project.id,
          name: project.name,
          color: project.color,
          organization_id: project.organization_id,
          sort_order: project.sort_order,
          created_at: project.created_at,
          updated_at: project.updated_at,
        };
        setProjects(prev => prev.map(p => p.id === tempId ? adaptedProject as unknown as LocalProject : p));
      }).catch(err => {
        console.error('Failed to create project:', err);
        setProjects(prev => prev.filter(p => p.id !== tempId));
      });

      return { data: tempProject, persisted: Promise.resolve(tempProject) };
    },
    []
  );

  const updateProject = useCallback(
    (id: string, changes: Partial<UpdateProjectRequest>): MutationResult => {
      const updates: { name?: string; color?: string; sort_order?: number } = {};
      if (changes.name !== undefined && changes.name !== null) updates.name = changes.name;
      if (changes.color !== undefined && changes.color !== null) updates.color = changes.color;
      if (changes.sort_order !== undefined && changes.sort_order !== null) updates.sort_order = changes.sort_order;

      setProjects(prev =>
        prev.map(p =>
          p.id === id ? { ...p, ...updates } as LocalProject : p
        )
      );

      kanbanApi.updateProject(id, updates).catch(err => {
        console.error('Failed to update project:', err);
      });

      return { persisted: Promise.resolve() };
    },
    []
  );

  const removeProject = useCallback(
    (id: string): MutationResult => {
      setProjects(prev => prev.filter(p => p.id !== id));

      kanbanApi.deleteProject(id).catch(err => {
        console.error('Failed to delete project:', err);
      });

      return { persisted: Promise.resolve() };
    },
    []
  );

  const value = useMemo<OrgContextValue>(
    () => ({
      organizationId,
      projects: projects as unknown as Project[],
      isLoading,
      error,
      retry: loadProjects,
      insertProject,
      updateProject,
      removeProject,
      getProject,
      projectsById: projectsById as Map<string, Project>,
      membersWithProfilesById: new Map(),
    }),
    [organizationId, projects, isLoading, error, loadProjects, insertProject, updateProject, removeProject, getProject, projectsById]
  );

  return <OrgContext.Provider value={value}>{children}</OrgContext.Provider>;
}
