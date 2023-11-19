import MetricService from '@/services/metric';
import MetricDetailDrawer from '../../metric-drawer';
import { MetricDefinition } from '@/types/bindings/metric-definition';
import MetricsPage from '../page';

async function getMetricDefinition(dbId: string) {
  try {
    const metricDefinition = await MetricService.getMetric(dbId);
    return metricDefinition;
  } catch (error) {
    console.error(error);
  }
}

const NEW_PATH = 'new';

export default async function MetricDetailLayout({
  children,
  params: { id: dbId },
}: {
  children: React.ReactNode;
  params: { id: string };
}) {
  let metricDefinition: MetricDefinition | undefined;
  if (dbId && dbId !== NEW_PATH) {
    metricDefinition = await getMetricDefinition(dbId);
    console.log({ metricDefinition });
  }
  return (
    <div className="relative flex h-full w-full">
      <MetricsPage />
      <MetricDetailDrawer metricDefinition={metricDefinition} />;
    </div>
  );
}
