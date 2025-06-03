import { test, expect } from '@playwright/test';
import { v4 as uuidv4 } from 'uuid';
import { AnalyticsEventSchema,AnalyticsEvent } from '../gen/messages_pb';
import { fromBinary } from '@bufbuild/protobuf';
const URL_TO_CAPTURE=['/api/event'];

function decodeEvent(data:Uint8Array){
  const event = fromBinary(AnalyticsEventSchema, data);
  return event;
}

test('test', async ({ page }) => {
  let requests:{url:string,data:AnalyticsEvent}[] = [];
  page.on('request', (request) => {
    if (URL_TO_CAPTURE.some(url => request.url().includes(url))) {
      console.log('>>',request.method(),request.url(),decodeEvent(request.postDataBuffer()!));
      requests.push({url:request.url(),data:decodeEvent(request.postDataBuffer()!)});
    }
  });
  page.on('response', (response) => {
    if (URL_TO_CAPTURE.some(url => response.url().includes(url))) {
      console.log('<<',response.status(),response.url(),response.body());
    }
  });
  await page.goto('http://localhost:1420/login');
  await page.getByRole('textbox', { name: 'Email' }).click();
  await page.getByRole('textbox', { name: 'Email' }).fill('kevin.yang.xgz@gmail.com');
  await page.getByRole('textbox', { name: 'Email' }).press('Tab');
  await page.getByRole('textbox', { name: 'Password' }).fill('test123456');
  await page.getByRole('button', { name: 'Login' }).click();
  await page.getByText('# bot chat').click();
  let msg=`Hello, this is the secret: ${uuidv4()}`;
  await page.getByRole('textbox', { name: 'Type a message...' }).fill(msg+'\n');
  await page.getByRole('button').nth(3).click();
  await expect(page.getByText(msg)).toBeVisible();
  
  //take a screenshot
  await page.screenshot({ path: './screenshots/'+msg+'.png' });
  expect(requests.length).toEqual(4);
  expect(requests[0].data.eventType.case).toEqual('appStart');
  expect(requests[1].data.eventType.case).toEqual('userLogin');
  expect(requests[2].data.eventType.case).toEqual('navigation');
  expect(requests[2].data.eventType.case).toEqual('navigation');
  expect(requests[3].data.eventType.case).toEqual('messageSent');
});