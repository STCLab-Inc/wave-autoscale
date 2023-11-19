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
  console.log(dbId);
  if (dbId === NEW_PATH) {
    return (
      <div className="relative flex h-full w-full">
        <ScalingComponentsPage />
      </div>
    );
  }

  try {
    const componentDefinition =
      await ScalingComponentService.getScalingComponent(dbId);
    console.log({ componentDefinition });

    return (
      <div className="relative flex h-full w-full">
        <ScalingComponentsPage />
        <ScalingComponentDetailDrawer
          componentDefinition={componentDefinition}
        />
      </div>
    );
  } catch (error) {
    console.error(error);
    return (
      <div className="relative flex h-full w-full">
        <ScalingComponentsPage />
      </div>
    );
  }
}
