import MetricService from '@/services/metric';
import MetricDetailDrawer from '../../metric-drawer';
import MetricsPage from '../page';

const NEW_PATH = 'new';

export default async function MetricDetailLayout({
  children,
  params: { id: dbId },
}: {
  children: React.ReactNode;
  params: { id: string };
}) {
  if (dbId === NEW_PATH) {
    return (
      <div className="relative flex h-full w-full">
        <MetricsPage />
      </div>
    );
  }

  try {
    const metricDefinition = await MetricService.getMetric(dbId);
    console.log({ metricDefinition });

    return (
      <div className="relative flex h-full w-full">
        <MetricsPage />
        <MetricDetailDrawer metricDefinition={metricDefinition} />
      </div>
    );
  } catch (error) {
    console.error(error);
    return (
      <div className="relative flex h-full w-full">
        <MetricsPage />
      </div>
    );
  }
}
