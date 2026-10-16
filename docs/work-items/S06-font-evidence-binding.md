# S06: 개발 후보와 폰트 원검증 증거의 정확 연결

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root scripts/windows_font_package_evidence.py·test_windows_font_package_evidence.py·.github/workflows/ci.yml·evidence·문서통합; soak_recovery_audit /tmp 초안
- 선행 작업: readonly8f21 개발 후보 및 폰트 원검증 완료

## 사용자 결과

다른 빌드의 폰트 증거를 현재 후보 검증으로 잘못 연결하지 않도록 거절한다.

## 계약과 범위

기존 immutable package/폰트 검증을 재사용하는 bounded adapter다. supplied PE/ZIP/원proof의 SHA와 네 payload·공지문을 정확히 연결한다. Cargo/schema/기존 패키징 변경은 root 검토 전 수행하지 않는다. 법적 완비·release false를 자동 승격하지 않는다.

## 인수 조건

맞는 snapshot 수락, currentce7efd에 과거8f21 proof 연결 거절, 파일변조·누락·다른 ZIP·잘못된 성공 상태 실패 경로를 검증한다. 과거 원자료는 보존한다.

## 검증과 handoff

/tmp 초안 준비 단계. 실제 artifact 대량 읽기/시험은 CPU 측정 종료 후 직렬 수행한다. 구현 전 완료를 표시하지 않는다.

### 구현·검증 진행

stdlib adapter/test와CI정의를root가추가했다. synthetic9시험은PASS이고실제8f21 ZIP/PE/폰트원result/4명령proof를연결해historical-binding.json생성에성공했다. 실제currentce7efd에과거8f21 proof연결은거절되고output미생성을확인했다. 원fontproof/verifier/ZIP는수정하지않았다. duplicateJSON/oversizedproof/ZIPentryearlybudget와false승격거절을포함한다. CLIcreate-only/추가독립검토는후속이며remoteCI/법적완비는미검증이다. 새런타임의존성/제품binary크기비용은없다.

### 재읽기 경계 교정과 CLI 검증

독립 검토가 SHA 검증 후 파일을 다시 읽어 파싱하는 경계를 지적했다. 검증한 raw bytes를 duplicate-key 거절 decoder에 직접 전달하도록 교정하고 bind의 실패 proof를 재읽기 없이 거절하는 회귀를 추가했다. 총11개 시험은 PASS다. 실제 CLI가 동일 입력으로 만든 두 historical8f binding은 SHA가 같고, 이미 있는 output에 다시 생성하는 시도는 거절하면서 원 bytes를 보존했다. 원CLI argv/exit/stderr는 cli-result.json에 있다. 신규현재CPU의폰트/패키지증거로승격하지않는다.

### 통합 완료 범위

root가 최종 script/test SHA와 독립 검토를 대조했다. local11시험·실제원8f binding·현재다른빌드거절·CLI결정성/기존output보존을 검증했다. CI에는 이 시험의 정의만 연결했으며 정상Git/remoteCI실행/현재새binary폰트증거/법적완비는 별도다. 이 adapter는 supplied snapshot 핀을 연결하며 서명이나 공급 attestation을 만들지 않는다.
