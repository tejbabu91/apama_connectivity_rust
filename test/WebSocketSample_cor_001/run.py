# Sample PySys testcase
# Copyright (c) 2019-2020 Software AG, Darmstadt, Germany and/or Software AG USA Inc., Reston, VA, USA, and/or its subsidiaries and/or its affiliates and/or their licensors. 
# Use, reproduction, transfer, publication or disclosure is prohibited except as specifically provided for in your License Agreement with Software AG 

from pysys.constants import *
from apama.basetest import ApamaBaseTest
from apama.correlator import CorrelatorHelper
import websockets
import asyncio
import threading
import time

class PySysTest(ApamaBaseTest):

	async def create_websocket_client(self, uri):
		print('creating client', uri)
		self.websocket = await websockets.connect(uri, compression=None)
		print('created', self.websocket)

	async def send_n_messages(self, n):
		count = 0
		for i in range(n):
			msg = f'Message {i}'
			# print('sending msg', msg)
			await self.websocket.send(msg)
			count = count + 1
			if i%100 == 0:
				# print('sending count', count)
				# await asyncio.sleep(0.0001)
				pass
			# print('after send')
		print('sending complete, count', count)
		return count
	
	async def recv_n_messages(self, n):
		count = 0
		try:
			for i in range(n):
				# print('before recv')
				await self.websocket.recv()
				count = count + 1
				if i%100 == 0:
					# print('receiving count', count)
					pass
				# print('received msg')
		except Exception as e:
			print('exception in read', e, 'after count', count)
		return count

	async def start_send_recv(self, url, n):
		await self.create_websocket_client(url)
		s = asyncio.create_task(self.send_n_messages(n))
		r = asyncio.create_task(self.recv_n_messages(n))
		return (await s, await r)

	def execute(self):
		correlator = CorrelatorHelper(self, name='mycorrelator')
		correlator.start(logfile='mycorrelator.log', config=[self.input+'/sample.yaml'], 
			configPropertyOverrides={'EXAMPLES_DIR':self.project.EXAMPLES_DIR, 'RUST_TARGET': self.project.RUST_TARGET})
		correlator.injectEPL(['ConnectivityPluginsControl.mon', 'ConnectivityPlugins.mon'], filedir=PROJECT.APAMA_HOME+'/monitors')
		correlator.injectEPL(filenames=[self.input+'/DemoApp.mon'])

		self.message_count = 100000

		res = asyncio.get_event_loop().run_until_complete(self.start_send_recv('ws://127.0.0.1:4999', self.message_count))
		s, r = res
		self.sent = s
		self.receved = r

		# self.waitForSignal('mycorrelator.log', expr="Got echo response", process=correlator, 
		# 	errorExpr=[' ERROR ', ' FATAL ', 'Failed to parse event'])

	def validate(self):
		# self.assertGrep('mycorrelator.log', expr=r'<connectivity\.diag\.rustTransport> (.*) Towards Host:')
		self.assertGrep('mycorrelator.log', expr='apamax.rust.RustTransportSample.*Receved: .*sec')
		self.assertThat('%s == %s', self.sent, self.message_count)
		self.assertThat('%s == %s', self.receved, self.message_count)
		# self.assertGrep('mycorrelator.out', expr='EchoTransport received message from host.*Hello to Rust from Apama')
	
