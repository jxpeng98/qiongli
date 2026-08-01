import {
  QiongliAppClient,
  type AppTransport
} from '@qiongli/app-api';

export function createValidatedAppClient(
  transport?: AppTransport
): QiongliAppClient {
  return new QiongliAppClient(transport);
}
