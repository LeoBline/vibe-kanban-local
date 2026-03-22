import { useState, useMemo, useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  PlusIcon,
  TrashIcon,
  PencilSimpleLineIcon,
  CheckIcon,
  XIcon,
} from '@phosphor-icons/react';
import { PrimaryButton } from '@vibe/ui/components/PrimaryButton';
import { useLocalProjectStore } from '@/shared/stores/useLocalProjectStore';
import { useLocalOrganizationStore } from '@/shared/stores/useLocalOrganizationStore';
import { InlineColorPicker } from '@vibe/ui/components/ColorPicker';
import { getRandomPresetColor } from '@/shared/lib/colors';
import { cn } from '@/shared/lib/utils';
import {
  SettingsCard,
  SettingsField,
} from './SettingsComponents';
import { DeleteRemoteProjectDialog } from '@/shared/dialogs/org/DeleteRemoteProjectDialog';

interface LocalProjectsSettingsSectionProps {
  initialState?: { organizationId?: string; projectId?: string };
}

interface LocalProjectItem {
  id: string;
  organization_id: string;
  name: string;
  color: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export function LocalProjectsSettingsSection({
  initialState,
}: LocalProjectsSettingsSectionProps) {
  const { t } = useTranslation('settings');

  const localOrganizations = useLocalOrganizationStore((state) => state.organizations);
  const fetchOrganizations = useLocalOrganizationStore((state) => state.fetchOrganizations);
  const createOrganization = useLocalOrganizationStore((state) => state.createOrganization);
  const setSelectedOrgId = useLocalOrganizationStore((state) => state.setSelectedOrgId);

  const localProjects = useLocalProjectStore((state) => state.projects);
  const createProject = useLocalProjectStore((state) => state.createProject);
  const updateProject = useLocalProjectStore((state) => state.updateProject);
  const deleteProject = useLocalProjectStore((state) => state.deleteProject);
  const fetchProjects = useLocalProjectStore((state) => state.fetchProjects);

  useEffect(() => {
    console.log('[LocalProjectsSettingsSection] Loading organizations...');
    fetchOrganizations();
  }, [fetchOrganizations]);

  const [selectedOrgId, setSelectedOrgIdLocal] = useState<string | null>(
    initialState?.organizationId ?? localOrganizations[0]?.id ?? null
  );
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(
    initialState?.projectId ?? null
  );
  const [editingProjectId, setEditingProjectId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState('');
  const [editingColor, setEditingColor] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [newProjectName, setNewProjectName] = useState('');
  const [newProjectColor, setNewProjectColor] = useState(getRandomPresetColor());
  const [error, setError] = useState<string | null>(null);

  const filteredProjects = useMemo(() => {
    if (!selectedOrgId) return [];
    return localProjects
      .filter((p) => p.organization_id === selectedOrgId)
      .sort((a, b) => a.sort_order - b.sort_order);
  }, [localProjects, selectedOrgId]);

  const handleOrgChange = useCallback((orgId: string) => {
    setSelectedOrgIdLocal(orgId);
    setSelectedOrgId(orgId);
    setSelectedProjectId(null);
    setEditingProjectId(null);
  }, [setSelectedOrgId]);

  const handleCreateOrganization = useCallback(async () => {
    const name = `Workspace ${localOrganizations.length + 1}`;
    const newOrg = await createOrganization(name);
    setSelectedOrgIdLocal(newOrg.id);
    setSelectedOrgId(newOrg.id);
  }, [localOrganizations.length, createOrganization, setSelectedOrgId]);

  const handleCreateProject = useCallback(async () => {
    if (!selectedOrgId) {
      setError('Please select or create an organization first');
      return;
    }

    const trimmedName = newProjectName.trim();
    if (!trimmedName) {
      setError('Project name is required');
      return;
    }

    const newProject = await createProject(selectedOrgId, trimmedName, newProjectColor);
    await fetchProjects(selectedOrgId);
    setSelectedProjectId(newProject.id);
    setIsCreating(false);
    setNewProjectName('');
    setNewProjectColor(getRandomPresetColor());
    setError(null);
  }, [selectedOrgId, newProjectName, newProjectColor, createProject, fetchProjects]);

  const handleStartEditing = useCallback((project: LocalProjectItem) => {
    setEditingProjectId(project.id);
    setEditingName(project.name);
    setEditingColor(project.color);
  }, []);

  const handleSaveEdit = useCallback(() => {
    if (!editingProjectId) return;

    const trimmedName = editingName.trim();
    if (!trimmedName) {
      setError('Project name is required');
      return;
    }

    updateProject(editingProjectId, { name: trimmedName, color: editingColor });
    setEditingProjectId(null);
    setError(null);
  }, [editingProjectId, editingName, editingColor, updateProject]);

  const handleCancelEdit = useCallback(() => {
    setEditingProjectId(null);
    setError(null);
  }, []);

  const handleDeleteProject = useCallback(async (project: LocalProjectItem) => {
    try {
      const result = await DeleteRemoteProjectDialog.show({
        projectName: project.name,
      });

      if (result === 'deleted') {
        deleteProject(project.id);
        if (selectedProjectId === project.id) {
          setSelectedProjectId(null);
        }
      }
    } catch {
      // Dialog cancelled
    }
  }, [deleteProject, selectedProjectId]);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium text-high mb-4">
          {t('settings.localProjects.title', 'Local Projects')}
        </h2>
        <p className="text-sm text-low mb-4">
          {t(
            'settings.localProjects.description',
            'Manage your local projects stored on this device.'
          )}
        </p>
      </div>

      <SettingsCard title={t('settings.localProjects.organization', 'Organization')}>
        <div className="space-y-4">
          <SettingsField
            label={t('settings.localProjects.organization', 'Organization')}
          >
            <div className="flex items-center gap-2">
              <select
                value={selectedOrgId ?? ''}
                onChange={(e) => handleOrgChange(e.target.value)}
                className="flex-1 px-3 py-2 rounded-md border border-border bg-background text-high"
              >
                <option value="">
                  {t('settings.localProjects.selectOrg', '-- Select --')}
                </option>
                {localOrganizations.map((org) => (
                  <option key={org.id} value={org.id}>
                    {org.name}
                  </option>
                ))}
              </select>
              <PrimaryButton
                variant="secondary"
                onClick={handleCreateOrganization}
              >
                <PlusIcon className="size-icon-xs mr-1" />
                {t('settings.localProjects.createOrg', 'New Org')}
              </PrimaryButton>
            </div>
          </SettingsField>
        </div>
      </SettingsCard>

      {selectedOrgId && (
        <>
          <SettingsCard title={t('settings.localProjects.projects', 'Projects')}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-base font-medium text-high">
                {t('settings.localProjects.projects', 'Projects')}
              </h3>
              {!isCreating && (
                <PrimaryButton
                  variant="secondary"
                  onClick={() => setIsCreating(true)}
                >
                  <PlusIcon className="size-icon-xs mr-1" />
                  {t('settings.localProjects.createProject', 'New Project')}
                </PrimaryButton>
              )}
            </div>

            {isCreating && (
              <div className="mb-4 p-4 bg-secondary rounded-md space-y-3">
                <div className="flex items-center gap-2">
                  <InlineColorPicker
                    value={newProjectColor}
                    onChange={setNewProjectColor}
                  />
                  <input
                    type="text"
                    value={newProjectName}
                    onChange={(e) => setNewProjectName(e.target.value)}
                    placeholder={t(
                      'settings.localProjects.projectNamePlaceholder',
                      'Project name'
                    )}
                    className="flex-1 px-3 py-2 rounded-md border border-border bg-background text-high"
                    autoFocus
                  />
                </div>
                {error && <p className="text-sm text-error">{error}</p>}
                <div className="flex justify-end gap-2">
                  <button
                    type="button"
                    onClick={() => {
                      setIsCreating(false);
                      setNewProjectName('');
                      setError(null);
                    }}
                    className="px-3 py-2 text-sm text-low hover:text-high"
                  >
                    {t('common.cancel', 'Cancel')}
                  </button>
                  <PrimaryButton onClick={handleCreateProject}>
                    <CheckIcon className="size-icon-xs mr-1" />
                    {t('common.create', 'Create')}
                  </PrimaryButton>
                </div>
              </div>
            )}

            {filteredProjects.length === 0 && !isCreating ? (
              <p className="text-sm text-low py-4 text-center">
                {t('settings.localProjects.noProjects', 'No projects yet')}
              </p>
            ) : (
              <div className="space-y-2">
                {filteredProjects.map((project) => (
                  <div
                    key={project.id}
                    className={cn(
                      'flex items-center gap-3 p-3 rounded-md',
                      selectedProjectId === project.id
                        ? 'bg-brand/10 border border-brand/30'
                        : 'bg-secondary hover:bg-secondary/80',
                      editingProjectId === project.id && 'ring-2 ring-brand'
                    )}
                  >
                    {editingProjectId === project.id ? (
                      <>
                        <InlineColorPicker
                          value={editingColor}
                          onChange={setEditingColor}
                        />
                        <input
                          type="text"
                          value={editingName}
                          onChange={(e) => setEditingName(e.target.value)}
                          className="flex-1 px-2 py-1 rounded border border-border bg-background text-high"
                          autoFocus
                          onKeyDown={(e) => {
                            if (e.key === 'Enter') handleSaveEdit();
                            if (e.key === 'Escape') handleCancelEdit();
                          }}
                        />
                        <button
                          type="button"
                          onClick={handleSaveEdit}
                          className="p-1 text-success hover:bg-success/10 rounded"
                        >
                          <CheckIcon className="size-icon-xs" />
                        </button>
                        <button
                          type="button"
                          onClick={handleCancelEdit}
                          className="p-1 text-low hover:bg-secondary rounded"
                        >
                          <XIcon className="size-icon-xs" />
                        </button>
                      </>
                    ) : (
                      <>
                        <div
                          className="size-dot rounded-full shrink-0"
                          style={{ backgroundColor: `hsl(${project.color})` }}
                        />
                        <span
                          className="flex-1 text-sm text-high cursor-pointer"
                          onClick={() => setSelectedProjectId(project.id)}
                        >
                          {project.name}
                        </span>
                        <button
                          type="button"
                          onClick={() => handleStartEditing(project)}
                          className="p-1 text-low hover:text-high hover:bg-secondary rounded"
                          title={t('common.edit', 'Edit')}
                        >
                          <PencilSimpleLineIcon className="size-icon-xs" />
                        </button>
                        <button
                          type="button"
                          onClick={() => handleDeleteProject(project)}
                          className="p-1 text-low hover:text-error hover:bg-error/10 rounded"
                          title={t('common.delete', 'Delete')}
                        >
                          <TrashIcon className="size-icon-xs" />
                        </button>
                      </>
                    )}
                  </div>
                ))}
              </div>
            )}
          </SettingsCard>
        </>
      )}
    </div>
  );
}
