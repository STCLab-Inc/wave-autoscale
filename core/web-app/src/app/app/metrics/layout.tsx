import Link from 'next/link';
import ContentHeader from '../content-header';
import MetricService from '@/services/metric';
import { MetricDefinition } from '@/types/bindings/metric-definition';

async function getMetrics() {
  try {
    const metrics = await MetricService.getMetrics();
    return metrics;
  } catch (e) {
    console.error(e);
  }
  return [];
}

export default async function MetricsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const metrics = await getMetrics();
  console.log({ metrics });
  return (
    <div className="flex h-full w-full">
      <main className="relative flex flex-1 flex-col">
        <div>
          <ContentHeader
            title="Metrics"
            right={
              <div className="flex items-center">
                <Link href="/app/metrics/new" className="no-underline">
                  <button className="btn-primary btn-sm btn">Add metric</button>
                </Link>
              </div>
            }
          />
          <div className="w-full overflow-x-auto">
            <table className="table-compact table w-full">
              {/* head */}
              <thead>
                <tr>
                  <th>
                    <label>
                      <input type="checkbox" className="checkbox" />
                    </label>
                  </th>
                  <th>Metric Kind</th>
                  <th>ID</th>
                  <th>Metadata</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {metrics.map((metric: MetricDefinition) => {
                  const metadata = Object.keys(metric.metadata)
                    .sort()
                    .map((key) => (
                      <div key={key}>
                        <span className="font-bold">{key}</span>:{' '}
                        {(metric.metadata as any)[key]}
                      </div>
                    ));
                  return (
                    <tr key={metric.db_id}>
                      <th>
                        <label>
                          <input type="checkbox" className="checkbox" />
                        </label>
                      </th>
                      <td>{metric.metric_kind}</td>
                      <td>{metric.id}</td>
                      <td>{metadata}</td>
                      <td>
                        <Link href={`/app/metrics/${metric.db_id}`}>
                          <button className="btn-primary btn-xs btn">
                            details
                          </button>
                        </Link>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
              {/* foot */}
              <tfoot>
                <tr>
                  <th></th>
                  <th></th>
                  <th></th>
                  <th></th>
                  <th></th>
                </tr>
              </tfoot>
            </table>
          </div>
        </div>
        {children}
      </main>
    </div>
  );
}
