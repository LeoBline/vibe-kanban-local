import { useMemo } from 'react';
import { useShape } from '@/shared/integrations/electric/hooks';
import { PROJECTS_SHAPE } from 'shared/remote-types';
import { useLocalProjectStore } from '@/shared/stores/useLocalProjectStore';

const LOCAL_ORG_PREFIX = 'local-';

function isLocalOrganization(organizationId: string | null): boolean {
  return !!organizationId && organizationId.startsWith(LOCAL_ORG_PREFIX);
}

export function useOrganizationProjects(organizationId: string | null) {
  const allLocalProjects = useLocalProjectStore((state) => {
    console.log('[DEBUG useOrganizationProjects] store subscription', { 
      organizationId,
      projectsCount: state.projects.length,
      projectIds: state.projects.map(p => p.id)
    });
    return state.projects;
  });
  const isLocalOrg = isLocalOrganization(organizationId);
  const enabled = !!organizationId;

  const { data: remoteData, isLoading: isRemoteLoading, error } = useShape(
    PROJECTS_SHAPE,
    { organization_id: organizationId || '' },
    { enabled: enabled && !isLocalOrg }
  );

  const localProjects = useMemo(() => {
    if (!organizationId || !isLocalOrg) return [];
    console.log('[DEBUG useOrganizationProjects] comparing', { 
      searchOrgId: organizationId, 
      searchOrgIdType: typeof organizationId,
      targetProjectId: 'local-project-1774184911523-q608kry',
      allProjectsOrgIds: allLocalProjects.map(p => ({ id: p.id, orgId: p.organization_id, orgIdType: typeof p.organization_id })),
    });
    const filtered = allLocalProjects
      .filter((p) => p.organization_id === organizationId)
      .sort((a, b) => a.sort_order - b.sort_order);
    console.log('[DEBUG useOrganizationProjects] filtered projects', { 
      organizationId, 
      allProjectsCount: allLocalProjects.length,
      filteredCount: filtered.length,
      projectIds: filtered.map(p => p.id),
      targetFound: filtered.some(p => p.id === 'local-project-1774184911523-q608kry')
    });
    return filtered;
  }, [organizationId, isLocalOrg, allLocalProjects]);

  const data = isLocalOrg ? (localProjects as unknown as typeof remoteData) : remoteData;
  const isLoading = isLocalOrg ? false : isRemoteLoading;

  return {
    data,
    isLoading,
    isError: !!error,
    error,
  };
}
