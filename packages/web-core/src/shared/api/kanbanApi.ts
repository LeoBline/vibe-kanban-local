import { fetchApi } from './client';
import type { IssuePriority, Tag } from 'shared/remote-types';

export type { IssuePriority, Tag };

export interface LocalOrganization {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface LocalProject {
  id: string;
  organization_id: string;
  name: string;
  color: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface LocalProjectStatus {
  id: string;
  project_id: string;
  name: string;
  color: string;
  sort_order: number;
  hidden: number;
  created_at: string;
}

export interface LocalTag {
  id: string;
  project_id: string;
  name: string;
  color: string;
  created_at: string;
}

export interface LocalIssue {
  id: string;
  project_id: string;
  issue_number: number;
  simple_id: string;
  status_id: string;
  title: string;
  description?: string | null;
  priority: IssuePriority | null;
  sort_order: number;
  start_date?: string | null;
  target_date?: string | null;
  completed_at?: string | null;
  parent_issue_id?: string | null;
  parent_issue_sort_order?: number | null;
  extension_metadata?: string;
  creator_user_id?: string | null;
  created_at: string;
  updated_at: string;
}

export const kanbanApi = {
  async listOrganizations(): Promise<LocalOrganization[]> {
    const response = await fetchApi('/api/local/organizations');
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to fetch organizations');
    return data.data;
  },

  async createOrganization(name: string): Promise<LocalOrganization> {
    const response = await fetchApi('/api/local/organizations', {
      method: 'POST',
      body: JSON.stringify({ name }),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to create organization');
    return data.data;
  },

  async getOrganization(id: string): Promise<LocalOrganization> {
    const response = await fetchApi(`/api/local/organizations/${id}`);
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to fetch organization');
    return data.data;
  },

  async updateOrganization(id: string, name: string): Promise<LocalOrganization> {
    const response = await fetchApi(`/api/local/organizations/${id}`, {
      method: 'PUT',
      body: JSON.stringify({ name }),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to update organization');
    return data.data;
  },

  async deleteOrganization(id: string): Promise<void> {
    const response = await fetchApi(`/api/local/organizations/${id}`, {
      method: 'DELETE',
    });
    if (!response.ok) {
      const data = await response.json();
      throw new Error(data.error || 'Failed to delete organization');
    }
  },

  async listProjects(organizationId: string): Promise<LocalProject[]> {
    const response = await fetchApi(`/api/local/projects?organization_id=${encodeURIComponent(organizationId)}`);
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to fetch projects');
    return data.data;
  },

  async createProject(organizationId: string, name: string, color: string, id?: string): Promise<LocalProject> {
    const response = await fetchApi('/api/local/projects', {
      method: 'POST',
      body: JSON.stringify({ organization_id: organizationId, name, color, id }),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to create project');
    return data.data;
  },

  async getProject(id: string): Promise<LocalProject> {
    const response = await fetchApi(`/api/local/projects/${id}`);
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to fetch project');
    return data.data;
  },

  async updateProject(id: string, updates: { name?: string; color?: string; sort_order?: number }): Promise<LocalProject> {
    const response = await fetchApi(`/api/local/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to update project');
    return data.data;
  },

  async deleteProject(id: string): Promise<void> {
    const response = await fetchApi(`/api/local/projects/${id}`, {
      method: 'DELETE',
    });
    if (!response.ok) {
      const data = await response.json();
      throw new Error(data.error || 'Failed to delete project');
    }
  },

  async listStatuses(projectId: string): Promise<LocalProjectStatus[]> {
    const response = await fetchApi(`/api/local/statuses?project_id=${encodeURIComponent(projectId)}`);
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to fetch statuses');
    return data.data;
  },

  async createStatus(projectId: string, name: string, color: string, sortOrder: number, hidden: boolean): Promise<LocalProjectStatus> {
    const response = await fetchApi('/api/local/statuses', {
      method: 'POST',
      body: JSON.stringify({
        project_id: projectId,
        name,
        color,
        sort_order: sortOrder,
        hidden,
      }),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to create status');
    return data.data;
  },

  async getStatus(id: string): Promise<LocalProjectStatus> {
    const response = await fetchApi(`/api/local/statuses/${id}`);
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to fetch status');
    return data.data;
  },

  async updateStatus(id: string, updates: { name?: string; color?: string; sort_order?: number; hidden?: boolean }): Promise<LocalProjectStatus> {
    const response = await fetchApi(`/api/local/statuses/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to update status');
    return data.data;
  },

  async deleteStatus(id: string): Promise<void> {
    const response = await fetchApi(`/api/local/statuses/${id}`, {
      method: 'DELETE',
    });
    if (!response.ok) {
      const data = await response.json();
      throw new Error(data.error || 'Failed to delete status');
    }
  },

  async createDefaultStatuses(projectId: string): Promise<LocalProjectStatus[]> {
    const response = await fetchApi(`/api/local/statuses/default/${encodeURIComponent(projectId)}`, {
      method: 'POST',
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to create default statuses');
    return data.data;
  },

  async listTags(projectId: string): Promise<LocalTag[]> {
    const response = await fetchApi(`/api/local/tags?project_id=${encodeURIComponent(projectId)}`);
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to fetch tags');
    return data.data;
  },

  async createTag(projectId: string, name: string, color: string): Promise<LocalTag> {
    const response = await fetchApi('/api/local/tags', {
      method: 'POST',
      body: JSON.stringify({
        project_id: projectId,
        name,
        color,
      }),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to create tag');
    return data.data;
  },

  async getTag(id: string): Promise<LocalTag> {
    const response = await fetchApi(`/api/local/tags/${id}`);
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to fetch tag');
    return data.data;
  },

  async updateTag(id: string, updates: { name?: string; color?: string }): Promise<Tag> {
    const response = await fetchApi(`/api/local/tags/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to update tag');
    return data.data;
  },

  async deleteTag(id: string): Promise<void> {
    const response = await fetchApi(`/api/local/tags/${id}`, {
      method: 'DELETE',
    });
    if (!response.ok) {
      const data = await response.json();
      throw new Error(data.error || 'Failed to delete tag');
    }
  },

  async listIssues(projectId: string): Promise<LocalIssue[]> {
    const response = await fetchApi(`/api/local/issues?project_id=${encodeURIComponent(projectId)}`);
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to fetch issues');
    return data.data;
  },

  async createIssue(issue: {
    project_id: string;
    status_id: string;
    title: string;
    description?: string;
    priority?: string;
    sort_order?: number;
    start_date?: string;
    target_date?: string;
  }): Promise<LocalIssue> {
    const response = await fetchApi('/api/local/issues', {
      method: 'POST',
      body: JSON.stringify(issue),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to create issue');
    return data.data;
  },

  async getIssue(id: string): Promise<LocalIssue> {
    const response = await fetchApi(`/api/local/issues/${id}`);
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to fetch issue');
    return data.data;
  },

  async updateIssue(id: string, updates: {
    status_id?: string;
    title?: string;
    description?: string;
    priority?: string;
    sort_order?: number;
    start_date?: string;
    target_date?: string;
  }): Promise<LocalIssue> {
    const response = await fetchApi(`/api/local/issues/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to update issue');
    return data.data;
  },

  async deleteIssue(id: string): Promise<void> {
    const response = await fetchApi(`/api/local/issues/${id}`, {
      method: 'DELETE',
    });
    if (!response.ok) {
      const data = await response.json();
      throw new Error(data.error || 'Failed to delete issue');
    }
  },

  async bulkUpdateIssues(updates: Array<{
    id: string;
    changes: {
      status_id?: string;
      title?: string;
      description?: string;
      priority?: string;
      sort_order?: number;
      start_date?: string;
      target_date?: string;
    };
  }>): Promise<LocalIssue[]> {
    const response = await fetchApi('/api/local/issues/bulk', {
      method: 'POST',
      body: JSON.stringify({
        updates: updates.map((u) => ({ id: u.id, changes: u.changes })),
      }),
    });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || 'Failed to bulk update issues');
    return data.data;
  },
};
