APAMA CONNECTIVITY RUST

   A library to enable writing Apama Connectivity plug-ins in Rust.

	Currently a proof-of-concept; APIs are likely to change a lot.

DESCRIPTION

  This project aims to provide a Rust package which can be used as a base
  to write Apama Connectivity plug-ins completely in Rust.

LICENSE

Copyright 2019-2020 Software AG, Darmstadt, Germany and/or its licensors

   SPDX-License-Identifier: Apache-2.0

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.    

BUILD
	Install Rust (e.g. 1.42.0) and Apama (e.g. 10.5.2.0 core edition). Currently only linux (e.g. Ubuntu) and WSL is supported, it doesn't build on Windows yet. 

	There is no need to build at the top level, just build the individual transport(s) you want e.g. 

	> . apamainstalldir/Apama/bin/apama_env
   > cd examples/echo_transport
	> cargo build

TEST
	We provide some smoke tests in PySys (not yet comprehensive).

