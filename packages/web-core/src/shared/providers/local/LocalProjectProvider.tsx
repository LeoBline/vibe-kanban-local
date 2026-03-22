import { useState, useEffect, useMemo, useCallback, useRef, type ReactNode } from 'react';
import { ProjectContext, type ProjectContextValue } from '@/shared/hooks/useProjectContext';
import type { InsertResult, MutationResult } from '@/shared/lib/electric/types';
import type {
  CreateIssueRequest,
  UpdateIssueRequest,
  CreateProjectStatusRequest,
  UpdateProjectStatusRequest,
  CreateTagRequest,
  UpdateTagRequest,
  CreateIssueAssigneeRequest,
  CreateIssueFollowerRequest,
  CreateIssueTagRequest,
  CreateIssueRelationshipRequest,
  Issue,
  ProjectStatus,
  Tag,
  IssueAssignee,
  IssueFollower,
  IssueTag,
  IssueRelationship,
  PullRequest,
  Workspace,
} from 'shared/remote-types';
import { kanbanApi, type LocalIssue, LocalProjectStatus, LocalTag } from '@/shared/api/kanbanApi';

interface LocalProjectProviderProps {
  projectId: string;
  children: ReactNode;
}

export function LocalProjectProvider({ projectId, children }: LocalProjectProviderProps) {
  const [allIssues, setAllIssues] = useState<LocalIssue[]>([]);
  const [allStatuses, setAllStatuses] = useState<LocalProjectStatus[]>([]);
  const [allTags, setAllTags] = useState<LocalTag[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const currentProjectIdRef = useRef<string | null>(null);
  const initPromiseRef = useRef<Promise<void> | null>(null);

  const loadData = useCallback(async () => {
    const thisProjectId = projectId;
    
    if (currentProjectIdRef.current !== null && currentProjectIdRef.current !== thisProjectId && initPromiseRef.current) {
      console.log('[LocalProjectProvider] Project changed, resetting init state:', { oldProject: currentProjectIdRef.current, newProject: thisProjectId });
      currentProjectIdRef.current = null;
      initPromiseRef.current = null;
    }
    
    if (initPromiseRef.current) {
      console.log('[LocalProjectProvider] Already initializing for project:', thisProjectId, ', waiting...');
      return initPromiseRef.current;
    }

    currentProjectIdRef.current = thisProjectId;
    const initPromise = (async () => {
      try {
        setIsLoading(true);
        setError(null);

        console.log('[LocalProjectProvider] loadData called for project:', projectId);
        
        let existingStatuses = await kanbanApi.listStatuses(projectId);
        console.log('[LocalProjectProvider] Existing statuses count:', existingStatuses.length, existingStatuses.map(s => ({ id: s.id, name: s.name, projectId: s.project_id })));
        
        if (existingStatuses.length === 0) {
          console.log('[LocalProjectProvider] Creating default statuses for project:', projectId);
          const defaultStatuses = [
            { name: 'Backlog', color: '220 9% 46%', sort_order: 0, hidden: true },
            { name: 'To do', color: '217 91% 60%', sort_order: 1, hidden: false },
            { name: 'In progress', color: '38 92% 50%', sort_order: 2, hidden: false },
            { name: 'In review', color: '258 90% 66%', sort_order: 3, hidden: false },
            { name: 'Done', color: '142 71% 45%', sort_order: 4, hidden: false },
            { name: 'Cancelled', color: '0 84% 60%', sort_order: 5, hidden: true },
          ];

          const createdStatuses: LocalProjectStatus[] = [];
          for (const status of defaultStatuses) {
            try {
              console.log('[LocalProjectProvider] Creating status:', status.name, 'hidden:', status.hidden);
              const created = await kanbanApi.createStatus(projectId, status.name, status.color, status.sort_order, status.hidden);
              console.log('[LocalProjectProvider] Created status:', created.name, 'hidden:', created.hidden);
              createdStatuses.push(created);
            } catch (err) {
              console.error('[LocalProjectProvider] Failed to create default status:', status.name, err);
            }
          }
          console.log('[LocalProjectProvider] Created statuses count:', createdStatuses.length);
          if (createdStatuses.length > 0) {
            existingStatuses = createdStatuses;
          }
        }

        const [issuesData, tagsData] = await Promise.all([
          kanbanApi.listIssues(projectId),
          kanbanApi.listTags(projectId),
        ]);

        setAllIssues(issuesData);
        setAllTags(tagsData);
        
        console.log('[LocalProjectProvider] Checking project ID before setting statuses:', { currentProjectId: projectId, loadedForProjectId: existingStatuses[0]?.project_id });
        
        const relevantStatuses = existingStatuses.filter(s => s.project_id === projectId);
        console.log('[LocalProjectProvider] Relevant statuses for current project:', relevantStatuses.length);
        setAllStatuses(relevantStatuses);
      } catch (err) {
        console.error('Failed to load project data:', err);
        setError(err instanceof Error ? err : new Error('Failed to load project data'));
      } finally {
        setIsLoading(false);
      }
    })();
    
    initPromiseRef.current = initPromise;
    return initPromise;
  }, [projectId]);

  useEffect(() => {
    const abortController = new AbortController();
    loadData().then(() => {
      if (abortController.signal.aborted) {
        console.log('[LocalProjectProvider] Aborted stale initialization');
      } else {
        console.log('[LocalProjectProvider] Initialization complete for project:', currentProjectIdRef.current);
      }
    });
    return () => {
      abortController.abort();
    };
  }, [loadData]);

  const issues = useMemo(() => allIssues.filter(i => i.project_id === projectId), [allIssues, projectId]);
  const statuses = useMemo(() => allStatuses.filter(s => s.project_id === projectId), [allStatuses, projectId]);
  const tags = useMemo(() => allTags.filter(t => t.project_id === projectId), [allTags, projectId]);

  console.log('[LocalProjectProvider] Context statuses for project', projectId, ':', statuses.length, statuses.map(s => ({ id: s.id, name: s.name, hidden: s.hidden })));

  const issuesById = useMemo(() => {
    const map = new Map<string, Issue>();
    for (const issue of issues) {
      const adaptedIssue: Issue = {
        id: issue.id,
        project_id: issue.project_id,
        issue_number: issue.issue_number,
        simple_id: issue.simple_id,
        status_id: issue.status_id,
        title: issue.title,
        description: issue.description ?? null,
        priority: issue.priority ?? null,
        sort_order: issue.sort_order,
        start_date: issue.start_date ?? null,
        target_date: issue.target_date ?? null,
        completed_at: issue.completed_at ?? null,
        parent_issue_id: issue.parent_issue_id ?? null,
        parent_issue_sort_order: issue.parent_issue_sort_order ?? null,
        extension_metadata: issue.extension_metadata ?? null,
        creator_user_id: issue.creator_user_id ?? null,
        created_at: issue.created_at,
        updated_at: issue.updated_at,
      };
      map.set(issue.id, adaptedIssue);
    }
    return map;
  }, [issues]);

  const statusesById = useMemo(() => {
    const map = new Map<string, ProjectStatus>();
    for (const status of statuses) {
      const adaptedStatus: ProjectStatus = {
        id: status.id,
        project_id: status.project_id,
        name: status.name,
        color: status.color,
        sort_order: status.sort_order,
        hidden: !!status.hidden,
        created_at: status.created_at,
      };
      map.set(status.id, adaptedStatus);
    }
    return map;
  }, [statuses]);

  const tagsById = useMemo(() => {
    const map = new Map<string, Tag>();
    for (const tag of tags) {
      const adaptedTag: Tag = {
        id: tag.id,
        project_id: tag.project_id,
        name: tag.name,
        color: tag.color,
      };
      map.set(tag.id, adaptedTag);
    }
    return map;
  }, [tags]);

  const insertIssue = useCallback(
    (data: CreateIssueRequest): InsertResult<Issue> => {
      const tempId = `temp-${Date.now()}`;
      const now = new Date().toISOString();
      const tempIssue: Issue = {
        id: tempId,
        project_id: projectId,
        issue_number: 0,
        simple_id: '',
        status_id: data.status_id,
        title: data.title,
        description: data.description ?? null,
        priority: data.priority ?? null,
        sort_order: data.sort_order ?? 0,
        start_date: data.start_date ?? null,
        target_date: data.target_date ?? null,
        completed_at: null,
        parent_issue_id: null,
        parent_issue_sort_order: null,
        extension_metadata: null,
        creator_user_id: null,
        created_at: now,
        updated_at: now,
      };

      setAllIssues(prev => [...prev, tempIssue as unknown as LocalIssue]);

      kanbanApi.createIssue({
        project_id: projectId,
        status_id: data.status_id,
        title: data.title,
        description: data.description ?? undefined,
        priority: data.priority ?? undefined,
        sort_order: data.sort_order,
        start_date: data.start_date ?? undefined,
        target_date: data.target_date ?? undefined,
      }).then(newIssue => {
        const adaptedIssue: Issue = {
          id: newIssue.id,
          project_id: newIssue.project_id,
          issue_number: newIssue.issue_number,
          simple_id: newIssue.simple_id,
          status_id: newIssue.status_id,
          title: newIssue.title,
          description: newIssue.description ?? null,
          priority: newIssue.priority ?? null,
          sort_order: newIssue.sort_order,
          start_date: newIssue.start_date ?? null,
          target_date: newIssue.target_date ?? null,
          completed_at: newIssue.completed_at ?? null,
          parent_issue_id: newIssue.parent_issue_id ?? null,
          parent_issue_sort_order: newIssue.parent_issue_sort_order ?? null,
          extension_metadata: newIssue.extension_metadata ?? null,
          creator_user_id: newIssue.creator_user_id ?? null,
          created_at: newIssue.created_at,
          updated_at: newIssue.updated_at,
        };
        setAllIssues(prev => prev.map(i => i.id === tempId ? adaptedIssue as unknown as LocalIssue : i));
      }).catch(err => {
        console.error('Failed to create issue:', err);
        setAllIssues(prev => prev.filter(i => i.id !== tempId));
      });

      return { data: tempIssue, persisted: Promise.resolve(tempIssue) };
    },
    [projectId]
  );

  const updateIssue = useCallback(
    (id: string, changes: Partial<UpdateIssueRequest>): MutationResult => {
      const updates: Record<string, unknown> = {};
      if (changes.title !== undefined) updates.title = changes.title;
      if (changes.description !== undefined) updates.description = changes.description;
      if (changes.status_id !== undefined) updates.status_id = changes.status_id;
      if (changes.priority !== undefined) updates.priority = changes.priority;
      if (changes.sort_order !== undefined) updates.sort_order = changes.sort_order;
      if (changes.start_date !== undefined) updates.start_date = changes.start_date;
      if (changes.target_date !== undefined) updates.target_date = changes.target_date;

      setAllIssues(prev =>
        prev.map(issue =>
          issue.id === id ? { ...issue, ...updates } as LocalIssue : issue
        )
      );

      kanbanApi.updateIssue(id, updates).catch(err => {
        console.error('Failed to update issue:', err);
      });

      return { persisted: Promise.resolve() };
    },
    []
  );

  const removeIssue = useCallback(
    (id: string): MutationResult => {
      setAllIssues(prev => prev.filter(issue => issue.id !== id));

      kanbanApi.deleteIssue(id).catch(err => {
        console.error('Failed to delete issue:', err);
      });

      return { persisted: Promise.resolve() };
    },
    []
  );

  const insertStatus = useCallback(
    (data: CreateProjectStatusRequest): InsertResult<ProjectStatus> => {
      const tempId = `temp-status-${Date.now()}`;
      const now = new Date().toISOString();
      const tempStatus: ProjectStatus = {
        id: tempId,
        project_id: projectId,
        name: data.name,
        color: data.color,
        sort_order: data.sort_order ?? 0,
        hidden: data.hidden ?? false,
        created_at: now,
      };

      setAllStatuses(prev => [...prev, tempStatus as unknown as LocalProjectStatus]);

      kanbanApi.createStatus(
        projectId,
        data.name,
        data.color,
        data.sort_order ?? 0,
        data.hidden ?? false
      ).then(newStatus => {
        const adaptedStatus: ProjectStatus = {
          id: newStatus.id,
          project_id: newStatus.project_id,
          name: newStatus.name,
          color: newStatus.color,
          sort_order: newStatus.sort_order,
          hidden: !!newStatus.hidden,
          created_at: newStatus.created_at,
        };
        setAllStatuses(prev => prev.map(s => s.id === tempId ? adaptedStatus as unknown as LocalProjectStatus : s));
      }).catch(err => {
        console.error('Failed to create status:', err);
        setAllStatuses(prev => prev.filter(s => s.id !== tempId));
      });

      return { data: tempStatus, persisted: Promise.resolve(tempStatus) };
    },
    [projectId]
  );

  const updateStatus = useCallback(
    (id: string, changes: Partial<UpdateProjectStatusRequest>): MutationResult => {
      const updates: Record<string, unknown> = {};
      if (changes.name !== undefined) updates.name = changes.name;
      if (changes.color !== undefined) updates.color = changes.color;
      if (changes.sort_order !== undefined) updates.sort_order = changes.sort_order;
      if (changes.hidden !== undefined) updates.hidden = changes.hidden;

      setAllStatuses(prev =>
        prev.map(status =>
          status.id === id ? { ...status, ...updates } as LocalProjectStatus : status
        )
      );

      kanbanApi.updateStatus(id, updates).catch(err => {
        console.error('Failed to update status:', err);
      });

      return { persisted: Promise.resolve() };
    },
    []
  );

  const removeStatus = useCallback(
    (id: string): MutationResult => {
      setAllStatuses(prev => prev.filter(status => status.id !== id));

      kanbanApi.deleteStatus(id).catch(err => {
        console.error('Failed to delete status:', err);
      });

      return { persisted: Promise.resolve() };
    },
    []
  );

  const insertTag = useCallback(
    (data: CreateTagRequest): InsertResult<Tag> => {
      const tempId = `temp-tag-${Date.now()}`;
      const tempTag: Tag = {
        id: tempId,
        project_id: projectId,
        name: data.name,
        color: data.color,
      };

      setAllTags(prev => [...prev, tempTag as unknown as LocalTag]);

      kanbanApi.createTag(projectId, data.name, data.color).then(newTag => {
        const adaptedTag: Tag = {
          id: newTag.id,
          project_id: newTag.project_id,
          name: newTag.name,
          color: newTag.color,
        };
        setAllTags(prev => prev.map(t => t.id === tempId ? adaptedTag as unknown as LocalTag : t));
      }).catch(err => {
        console.error('Failed to create tag:', err);
        setAllTags(prev => prev.filter(t => t.id !== tempId));
      });

      return { data: tempTag, persisted: Promise.resolve(tempTag) };
    },
    [projectId]
  );

  const updateTag = useCallback(
    (id: string, changes: Partial<UpdateTagRequest>): MutationResult => {
      const updates: Record<string, unknown> = {};
      if (changes.name !== undefined) updates.name = changes.name;
      if (changes.color !== undefined) updates.color = changes.color;

      setAllTags(prev =>
        prev.map(tag =>
          tag.id === id ? { ...tag, ...updates } as LocalTag : tag
        )
      );

      kanbanApi.updateTag(id, updates).catch(err => {
        console.error('Failed to update tag:', err);
      });

      return { persisted: Promise.resolve() };
    },
    []
  );

  const removeTag = useCallback(
    (id: string): MutationResult => {
      setAllTags(prev => prev.filter(tag => tag.id !== id));

      kanbanApi.deleteTag(id).catch(err => {
        console.error('Failed to delete tag:', err);
      });

      return { persisted: Promise.resolve() };
    },
    []
  );

  const insertIssueAssignee = useCallback(
    (_data: CreateIssueAssigneeRequest): InsertResult<IssueAssignee> => {
      const emptyAssignee: IssueAssignee = {
        id: '',
        issue_id: '',
        user_id: '',
        assigned_at: new Date().toISOString(),
      };
      return { data: emptyAssignee, persisted: Promise.resolve(emptyAssignee) };
    },
    []
  );

  const removeIssueAssignee = useCallback((_id: string): MutationResult => {
    return { persisted: Promise.resolve() };
  }, []);

  const insertIssueFollower = useCallback(
    (_data: CreateIssueFollowerRequest): InsertResult<IssueFollower> => {
      const emptyFollower: IssueFollower = {
        id: '',
        issue_id: '',
        user_id: '',
      };
      return { data: emptyFollower, persisted: Promise.resolve(emptyFollower) };
    },
    []
  );

  const removeIssueFollower = useCallback((_id: string): MutationResult => {
    return { persisted: Promise.resolve() };
  }, []);

  const insertIssueTag = useCallback(
    (_data: CreateIssueTagRequest): InsertResult<IssueTag> => {
      const emptyIssueTag: IssueTag = {
        id: '',
        issue_id: '',
        tag_id: '',
      };
      return { data: emptyIssueTag, persisted: Promise.resolve(emptyIssueTag) };
    },
    []
  );

  const removeIssueTag = useCallback((_id: string): MutationResult => {
    return { persisted: Promise.resolve() };
  }, []);

  const insertIssueRelationship = useCallback(
    (_data: CreateIssueRelationshipRequest): InsertResult<IssueRelationship> => {
      const emptyRelationship: IssueRelationship = {
        id: '',
        issue_id: '',
        related_issue_id: '',
        relationship_type: 'related' as const,
        created_at: new Date().toISOString(),
      };
      return { data: emptyRelationship, persisted: Promise.resolve(emptyRelationship) };
    },
    []
  );

  const removeIssueRelationship = useCallback((_id: string): MutationResult => {
    return { persisted: Promise.resolve() };
  }, []);

  const getIssue = (issueId: string) => issuesById.get(issueId);
  const getIssuesForStatus = (statusId: string): Issue[] => {
    const filtered = issues.filter(i => i.status_id === statusId);
    return filtered.map(issue => {
      const adaptedIssue: Issue = {
        id: issue.id,
        project_id: issue.project_id,
        issue_number: issue.issue_number,
        simple_id: issue.simple_id,
        status_id: issue.status_id,
        title: issue.title,
        description: issue.description ?? null,
        priority: issue.priority ?? null,
        sort_order: issue.sort_order,
        start_date: issue.start_date ?? null,
        target_date: issue.target_date ?? null,
        completed_at: issue.completed_at ?? null,
        parent_issue_id: issue.parent_issue_id ?? null,
        parent_issue_sort_order: issue.parent_issue_sort_order ?? null,
        extension_metadata: issue.extension_metadata ?? null,
        creator_user_id: issue.creator_user_id ?? null,
        created_at: issue.created_at,
        updated_at: issue.updated_at,
      };
      return adaptedIssue;
    });
  };
  const getAssigneesForIssue = (_issueId: string): IssueAssignee[] => [];
  const getFollowersForIssue = (_issueId: string): IssueFollower[] => [];
  const getTagsForIssue = (_issueId: string): IssueTag[] => [];
  const getTagObjectsForIssue = (_issueId: string): Tag[] => [];
  const getRelationshipsForIssue = (_issueId: string): IssueRelationship[] => [];
  const getStatus = (statusId: string) => statusesById.get(statusId);
  const getTag = (tagId: string) => tagsById.get(tagId);
  const getPullRequestsForIssue = (_issueId: string): PullRequest[] => [];
  const getWorkspacesForIssue = (_issueId: string): Workspace[] => [];

  const value = useMemo<ProjectContextValue>(
    () => ({
      projectId,
      issues: issues as unknown as Issue[],
      statuses: statuses as unknown as ProjectStatus[],
      tags: tags as unknown as Tag[],
      issueAssignees: [],
      issueFollowers: [],
      issueTags: [],
      issueRelationships: [],
      pullRequests: [],
      workspaces: [],
      isLoading,
      error,
      retry: loadData,
      insertIssue,
      updateIssue,
      removeIssue,
      insertStatus,
      updateStatus,
      removeStatus,
      insertTag,
      updateTag,
      removeTag,
      insertIssueAssignee,
      removeIssueAssignee,
      insertIssueFollower,
      removeIssueFollower,
      insertIssueTag,
      removeIssueTag,
      insertIssueRelationship,
      removeIssueRelationship,
      getIssue,
      getIssuesForStatus,
      getAssigneesForIssue,
      getFollowersForIssue,
      getTagsForIssue,
      getTagObjectsForIssue,
      getRelationshipsForIssue,
      getStatus,
      getTag,
      getPullRequestsForIssue,
      getWorkspacesForIssue,
      issuesById: issuesById as Map<string, Issue>,
      statusesById: statusesById as Map<string, ProjectStatus>,
      tagsById: tagsById as Map<string, Tag>,
    }),
    [
      projectId,
      issues,
      statuses,
      tags,
      issuesById,
      statusesById,
      tagsById,
      isLoading,
      error,
      loadData,
      insertIssue,
      updateIssue,
      removeIssue,
      insertStatus,
      updateStatus,
      removeStatus,
      insertTag,
      updateTag,
      removeTag,
      insertIssueAssignee,
      removeIssueAssignee,
      insertIssueFollower,
      removeIssueFollower,
      insertIssueTag,
      removeIssueTag,
      insertIssueRelationship,
      removeIssueRelationship,
    ]
  );

  return <ProjectContext.Provider value={value}>{children}</ProjectContext.Provider>;
}
