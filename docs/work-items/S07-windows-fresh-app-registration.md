# S07: 원 준비 묶음의 개발용 설치 등록 결합

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — app_install.rs 공유 export·문서·통합·native; diagnostic_cache — 새 app_install/registration_schema.rs typed bounded record/seal; soak_recovery_audit — 새 app_install/registration.rs 원cap coordinator; metrics_recovery — /tmp 실제 fixture 초안
- 선행 작업: fresh prepared discard scoped DONE, source555 whole4/40회귀 및 실제4bundles·기존prepare/CLIcontrol 호환

## 사용자 결과

이번 실행이 새로 만든 준비 묶음만 consume하여 원 root/payload/ready와 새 registration record·seal 원 IDs를 durable bytes로 결합한다. 개발용 등록이며 active installation·publisher trust·cold 삭제권한을 선언하지 않는다.

## 계약과 범위

별도 create-only install-registration.json(64KiB) 및 install-registration-seal.json(4KiB)을 원 root HANDLE로 생성한다. 두 원ID를 먼저 얻고 자기digest 없는 record의 canonical bytes를 domain SHA로 seal에 결합한다. nonce는 기존 Windows BCryptGenRandom32B로 생성하며 인증비밀이 아니다. 원payload/marker/전체5namespace·ACL/ID/hash를 cancel/deadline 확인하며 검증한다. 실패에는 원 PreparedBundle과 새caps를 보존하고 자동삭제/recreate/adopt하지 않는다. register_fresh, FreshRegistration.verify_current 및 typed RegistrationFailure만 우선 연결한다. 재시작 readonly·coldremove는 별도후속이다. 새dependency/lock/기존schema변경없음.

## 인수 조건

원syntheticPE+notice 등록·원 record/seal ID결합·bounded JSON·형식/unknown/trust/domain/nonce/nativeID/digest 거절; 원source/미디어/sentinel 보존; cancellation/deadline/midcreate-write-sync fault에서 모든원caps 보존; extraforeign/changedoriginal 거절. 실제Windows전체4검사와 immutable actualCPU fixture source/hash/prepost/privateJob0를 root독립검증한다.

## 검증과 handoff

구현 진행 중. 디스크 record/hash만으로 동일계정 공격방어·역사적권한·coldadopt를 증명하지 않는다. active pointer/registry/PATH/서명/updater/cleanOS는 이 수직단위의 인수가 아니다. ADR0005 Proposed를 유지한다.

통합 기록: 원 ready marker ID를 RegistrationRecord.ready_identity로 추가해 같은 bytes의 marker 교체도 구분한다. Schema의 record/seal IDs와 original cap IDs를 직접 대조한다. RegistrationFailure의 retained_file_cap_count는 신규 확인cap만 뜻하고 mutation_may_have_occurred는 첫 nativecreate시도 전에 conservativetrue다(내부create후검증실패를 cap0=생성0으로 해석하지 않는다). 첫source558 whole fmtPASS/ClippyFAIL은 fixture unused import로 보존했다. import수정 whole검사중 독립검토가 devfalse bundle의 선행거절 누락을 발견했고 변경0 거절 회귀·수정을 추가한 뒤 최종인수한다. 현재 등록전체 검증 미완료다.

선행거절 native RED: 원 devfalse PreparedBundle을 실제생성해 register_fresh를 호출한 시험은 mutationflag true에 실패했다(exit101/자연Job0/privateinput 불변). 이 결과를 expected red로 별도보존했다. 수정은 verify_prepared의 초기 devtrue gate이며 같은 회귀는 생성0/원bytes·marker·sentinel보존을 요구한다. 수정 포함 source558 전체4검사 진행 중이다. [원 실패](../validation/evidence/windows-app-registration-devflag-red-20261006-continue/root-review.json).

최종 scoped 인수: source558 Windows 전체fmt/Clippy/test/release4PASS·app_install49actual회귀·CPU4888f534…/7,917,568B(크기0). 실제CPU4freshbundles 정상등록/foreign·취소·기한 preflight변경0·원caps preserve·caller source cursor17·Drop후전체5namespace/payload/ready/record/sealbytes보존 PASS. record1414B/seal565B, bounded domain-separated sealSHA를 root와독립관측기에서재계산. 별도원Windows observer에서 생성프로세스종료후 root+5file actualNativeIDs/ulongvolume/hash/size를원record와대조해PASS. colddelete/activeinstallation/publishertrust 완료가 아니다.

증거: [최종 전체](../validation/evidence/windows-app-registration-preflight-workspace-20261006-continue/root-review.json), [실제CPU등록](../validation/evidence/windows-app-fresh-registration-real-cpu-20261006-continue/root-review.json), [별도 nativeID](../validation/evidence/windows-app-registration-native-identities-20261006-continue/root-review.json). 다음은 [제품 읽기 재검증](S07-windows-registered-reopen.md)이다.
