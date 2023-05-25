import Link from 'next/link';
import ContentHeader from '../content-header';
import MetricService from '@/services/metric';
import { MetricDefinition } from '@/types/bindings/metric-definition';
import AutoscalingHistoryService from '@/services/autoscaling-history';
import dayjs, { Dayjs } from 'dayjs';
import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';
import { renderKeyValuePairsWithJson } from '../keyvalue-renderer';

async function getHistory(
  from: Dayjs = dayjs().subtract(7, 'days'),
  to: Dayjs = dayjs()
) {
  const history = await AutoscalingHistoryService.getHistoryByFromTo(from, to);
  return history;
}

export default async function AutoscalingHistoryLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const history = await getHistory();
  return (
    <div className="flex h-full w-full">
      <main className="relative flex flex-1 flex-col">
        <div>
          <ContentHeader title="Autoscaling History" />
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
                  <th>Plan ID</th>
                  <th>Plan Item</th>
                  <th>Metric Values</th>
                  <th>Metadata Values</th>
                  <th>Fail Message</th>
                  <th>Status</th>
                  <th>Date</th>
                </tr>
              </thead>
              <tbody>
                {history.map((historyItem: AutoscalingHistoryDefinition) => {
                  console.log({
                    date: historyItem.created_at,
                    type: typeof historyItem.created_at,
                  });
                  return (
                    <tr key={historyItem.id}>
                      <th>
                        <label>
                          <input type="checkbox" className="checkbox" />
                        </label>
                      </th>
                      <td>{historyItem.plan_id}</td>
                      <td>
                        {renderKeyValuePairsWithJson(
                          historyItem.plan_item_json
                        )}
                      </td>
                      <td>
                        {renderKeyValuePairsWithJson(
                          historyItem.metric_values_json
                        )}
                      </td>
                      <td>
                        {renderKeyValuePairsWithJson(
                          historyItem.metadata_values_json
                        )}
                      </td>
                      <td>
                        {historyItem.fail_message &&
                          renderKeyValuePairsWithJson(historyItem.fail_message)}
                      </td>
                      <td>
                        {historyItem.fail_message ? (
                          <div className="badge-error badge">Failed</div>
                        ) : (
                          <div className="badge-success badge">Success</div>
                        )}
                      </td>
                      <td>
                        {dayjs
                          .unix(historyItem.created_at)
                          .format('YYYY-MM-DD HH:mm:ss')}
                      </td>
                      {/* <td>
                        <Link href={`/app/metrics/${metric.db_id}`}>
                          <button className="btn-primary btn-xs btn">
                            details
                          </button>
                        </Link>
                      </td> */}
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
