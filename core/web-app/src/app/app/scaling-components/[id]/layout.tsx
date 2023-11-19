import ScalingComponentService from '@/services/scaling-component';
import ScalingComponentDetailDrawer from '../../scaling-component-drawer';
import ScalingComponentsPage from '../page';

const NEW_PATH = 'new';

export default async function ScalingComponentDetailLayout({
  children,
  params: { id: dbId },
}: {
  children: React.ReactNode;
  params: { id: string };
}) {
  let componentDefinition;

  if (dbId && dbId !== NEW_PATH) {
    try {
      componentDefinition = await ScalingComponentService.getScalingComponent(
        dbId
      );
      console.log({ componentDefinition });
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <div className="relative flex h-full w-full">
      <ScalingComponentsPage />
      <ScalingComponentDetailDrawer componentDefinition={componentDefinition} />
    </div>
  );
}
