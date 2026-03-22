import { useEffect } from 'react';
import { useUserSystem } from '@/shared/hooks/useUserSystem';
import { getFirstProjectDestination } from '@/shared/lib/firstProjectDestination';
import { useOrganizationStore } from '@/shared/stores/useOrganizationStore';
import { useUiPreferencesStore } from '@/shared/stores/useUiPreferencesStore';
import { useAppNavigation } from '@/shared/hooks/useAppNavigation';
import { useLocalOrganizationStore } from '@/shared/stores/useLocalOrganizationStore';

export function RootRedirectPage() {
  const { config, loading, loginStatus } = useUserSystem();
  const setSelectedOrgId = useOrganizationStore((s) => s.setSelectedOrgId);
  const appNavigation = useAppNavigation();
  const isLoggedIn = loginStatus?.status === 'loggedin';

  useEffect(() => {
    if (loading || !config) {
      return;
    }

    let isActive = true;
    void (async () => {
      if (!config.remote_onboarding_acknowledged) {
        appNavigation.goToOnboarding({ replace: true });
        return;
      }

      const { selectedOrgId, selectedProjectId } =
        useUiPreferencesStore.getState();

      let destination = await getFirstProjectDestination(
        setSelectedOrgId,
        selectedOrgId,
        selectedProjectId
      );

      if (!isActive) {
        return;
      }

      if (destination?.kind === 'project') {
        appNavigation.goToProject(destination.projectId, { replace: true });
        return;
      }

      if (destination?.kind === 'workspaces') {
        appNavigation.goToWorkspaces({ replace: true });
        return;
      }

      if (!destination && !isLoggedIn) {
        try {
          const localOrgStore = useLocalOrganizationStore.getState();
          const existingOrgs = localOrgStore.organizations;
          
          if (existingOrgs.length > 0) {
            localOrgStore.setSelectedOrgId(existingOrgs[0].id);
          } else {
            const newOrg = localOrgStore.createOrganization(
              'My Workspace',
              `workspace-${Date.now()}`
            );
            localOrgStore.setSelectedOrgId(newOrg.id);
          }
          
          if (!isActive) return;
          appNavigation.goToWorkspaces({ replace: true });
          return;
        } catch (error) {
          console.error('Failed to create default organization:', error);
        }
      }

      appNavigation.goToWorkspaces({ replace: true });
    })();

    return () => {
      isActive = false;
    };
  }, [appNavigation, config, loading, loginStatus?.status, setSelectedOrgId]);

  return (
    <div className="h-screen bg-primary flex items-center justify-center">
      <p className="text-low">Loading...</p>
    </div>
  );
}
