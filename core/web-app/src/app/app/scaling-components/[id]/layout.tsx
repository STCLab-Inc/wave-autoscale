import MetricService from '@/services/metric';
import ScalingComponentDetailDrawer from '../../scaling-component-drawer';
import { MetricDefinition } from '@/types/bindings/metric-definition';
import ScalingComponentService from '@/services/scaling-component';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';

async function getScalingComponentDefinition(dbId: string) {
  try {
    const componentDefinition =
      await ScalingComponentService.getScalingComponent(dbId);
    return componentDefinition;
  } catch (e) {
    console.error(e);
  }
}

const NEW_PATH = 'new';

export default async function ScalingComponentDetailLayout({
  children,
  params: { id: dbId },
}: {
  children: React.ReactNode;
  params: { id: string };
}) {
  let componentDefinition: ScalingComponentDefinition | undefined;
  if (dbId && dbId !== NEW_PATH) {
    componentDefinition = await getScalingComponentDefinition(dbId);
    console.log({ componentDefinition: componentDefinition });
  }
  return (
    <ScalingComponentDetailDrawer componentDefinition={componentDefinition} />
  );
}
