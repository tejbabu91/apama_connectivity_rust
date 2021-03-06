# Sample PySys testcase
# Copyright (c) 2019-2020 Software AG, Darmstadt, Germany and/or Software AG USA Inc., Reston, VA, USA, and/or its subsidiaries and/or its affiliates and/or their licensors. 
# Use, reproduction, transfer, publication or disclosure is prohibited except as specifically provided for in your License Agreement with Software AG 

from pysys.constants import *
from apama.basetest import ApamaBaseTest
from apama.correlator import CorrelatorHelper

class PySysTest(ApamaBaseTest):

	def execute(self):
		correlator = CorrelatorHelper(self, name='mycorrelator')
		correlator.start(logfile='mycorrelator.log', config=[self.input+'/sample.yaml'], 
			configPropertyOverrides={'EXAMPLES_DIR':self.project.EXAMPLES_DIR, 'RUST_TARGET': self.project.RUST_TARGET})
		correlator.injectEPL(['ConnectivityPluginsControl.mon', 'ConnectivityPlugins.mon'], filedir=PROJECT.APAMA_HOME+'/monitors')
		correlator.injectEPL(filenames=[self.input+'/DemoApp.mon'])

		self.waitForSignal('mycorrelator.log', expr="Got echo response", process=correlator, 
			errorExpr=[' ERROR ', ' FATAL ', 'Failed to parse event'])
		correlator.shutdown()

	def validate(self):
		self.assertGrep('mycorrelator.log', expr=r'<connectivity\.diag\.rustTransport> (.*) Towards Host:')
		self.assertGrep('mycorrelator.out', expr=r'EchoTransport received message from host.*Hello to Rust from Apama')

		self.assertGrep('mycorrelator.log', expr=r'apamax.rust.RustTransportSample .* Got echo response: apamax.rust.EchoResponse.*Hello to Rust from Apama')
		self.assertGrep('mycorrelator.log', expr=r'Got echo response: apamax.rust.EchoResponse[(]"Hello from Rust') # checks

		self.assertGrep('mycorrelator.out', expr=r'Creating transport with config.*String\("myConfigKey"\): String\("config string value"\)')
		self.assertGrep('mycorrelator.out', expr=r'EchoTransport shutdown done')
		self.assertGrep('mycorrelator.out', expr=r'EchoTransport Dropped')
