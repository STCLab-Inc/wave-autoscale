import ScalingComponentService from '@/services/scaling-component';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';

import ScalingComponentDetailDrawer from '../../scaling-component-drawer';
import ScalingComponentsPage from '../page';

const NEW_PATH = 'new';

async function getScalingComponentDefinition(dbId: string) {
  try {
    const scalingComponentDefinition =
      await ScalingComponentService.getScalingComponent(dbId);
    return scalingComponentDefinition;
  } catch (error) {
    console.error(error);
  }
}

export default async function ScalingComponentDetailLayout({
  children,
  params: { id: dbId },
}: {
  children: React.ReactNode;
  params: { id: string };
}) {
  let scalingComponentDefinition: ScalingComponentDefinition | undefined;
  if (dbId && dbId !== NEW_PATH) {
    scalingComponentDefinition = await getScalingComponentDefinition(dbId);
    console.log({ scalingComponentDefinition });
  }

  return (
    <div className="relative flex h-full w-full">
      <ScalingComponentsPage />
      <ScalingComponentDetailDrawer
        componentDefinition={scalingComponentDefinition}
      />
    </div>
  );
}
