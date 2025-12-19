# *******************************************************************************
# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
# *******************************************************************************
class ResultCode:
    """
    Test scenario exit codes.
    """

    SUCCESS = 0
    PANIC = 101
    SIGABRT = -6
    SIGKILL = -9
    SIGSEGV = -11
    SIGTERM = -15
