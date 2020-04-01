# Sample PySys testcase
# Copyright (c) 2019-2020 Software AG, Darmstadt, Germany and/or Software AG USA Inc., Reston, VA, USA, and/or its subsidiaries and/or its affiliates and/or their licensors. 
# Use, reproduction, transfer, publication or disclosure is prohibited except as specifically provided for in your License Agreement with Software AG 

from pysys.constants import *
from apama.basetest import ApamaBaseTest
from apama.correlator import CorrelatorHelper

class PySysTest(ApamaBaseTest):

	def execute(self):
		correlator = CorrelatorHelper(self, name='mycorrelator', port=15309)
		correlator.start(logfile='mycorrelator.log', config=[self.input+'/sample.yaml'], 
			configPropertyOverrides={'TEST_TRANSPORT_DIR':self.project.TEST_TRANSPORT_DIR})
		correlator.injectEPL(['ConnectivityPluginsControl.mon', 'ConnectivityPlugins.mon'], filedir=PROJECT.APAMA_HOME+'/monitors')
		correlator.injectEPL(filenames=[self.input+'/DemoApp.mon'])

		self.waitForSignal('mycorrelator.log', expr="Got echo response", process=correlator, 
			errorExpr=[' ERROR ', ' FATAL ', 'Failed to parse event'])

	def validate(self):
		self.assertGrep('mycorrelator.log', expr=r'<connectivity\.diag\.rustTransport> (.*) Towards Host:')
		self.assertGrep('mycorrelator.log', expr='apamax.rust.RustTransportSample .* Got echo response: apamax.rust.EchoResponse.*Hello to Rust from Apama')
		self.assertGrep('mycorrelator.out', expr='EchoTransport received message from host.*Hello to Rust from Apama')
	