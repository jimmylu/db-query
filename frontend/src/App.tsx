import { BrowserRouter, Routes, Route, Outlet } from 'react-router-dom';
import { Refine } from '@refinedev/core';
import { RefineKbar, RefineKbarProvider } from '@refinedev/kbar';
import {
  ErrorComponent,
  ThemedLayoutV2,
  ThemedSiderV2,
  useNotificationProvider,
} from '@refinedev/antd';
import { App as AntdApp, ConfigProvider } from 'antd';
import '@refinedev/antd/dist/reset.css';

import { dataProvider } from './providers/dataProvider';
import { Dashboard } from './pages/Dashboard';
import { DataExplorePage } from './pages/DataExplorePage';
import { CustomHeader } from './components/CustomHeader';
import { CustomTitle } from './components/CustomTitle';
import { AWSDynamicBreadcrumbs } from './components/AWSDynamicBreadcrumbs';
import { awsTheme } from './theme/aws-theme';

function App() {
  return (
    <BrowserRouter>
      <RefineKbarProvider>
        <ConfigProvider theme={awsTheme}>
          <AntdApp>
            <Refine
              dataProvider={dataProvider}
              notificationProvider={useNotificationProvider}
              resources={[
                {
                  name: 'connections',
                  list: '/',
                  meta: {
                    label: '数据集列表',
                  },
                },
                {
                  name: 'data-explore',
                  list: '/data-explore',
                  meta: {
                    label: '数据探索',
                  },
                },
              ]}
              options={{
                syncWithLocation: true,
                warnWhenUnsavedChanges: true,
              }}
            >
              <Routes>
                <Route
                  element={
                    <ThemedLayoutV2
                      Sider={() => <ThemedSiderV2 fixed Title={({ collapsed }) => <CustomTitle collapsed={collapsed} />} />}
                      Header={() => <CustomHeader />}
                      Title={({ collapsed }) => <CustomTitle collapsed={collapsed} />}
                    >
                      <AWSDynamicBreadcrumbs />
                      <Outlet />
                    </ThemedLayoutV2>
                  }
                >
                  <Route index element={<Dashboard />} />
                  <Route path="data-explore" element={<DataExplorePage />} />
                  <Route path="*" element={<ErrorComponent />} />
                </Route>
              </Routes>
              <RefineKbar />
            </Refine>
          </AntdApp>
        </ConfigProvider>
      </RefineKbarProvider>
    </BrowserRouter>
  );
}

export default App;

