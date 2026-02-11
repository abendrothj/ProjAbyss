#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "DivingBell.generated.h"

class UStaticMeshComponent;
class USpotLightComponent;
class UBoxComponent;

UCLASS()
class PROJABYSS_API ADivingBell : public AActor
{
	GENERATED_BODY()

public:
	ADivingBell();

protected:
	virtual void BeginPlay() override;

public:
	virtual void Tick(float DeltaTime) override;

	// -- COMPONENTS --
	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Diving Bell")
	UStaticMeshComponent* BellMesh;

	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Diving Bell")
	USpotLightComponent* BellLight;

	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Diving Bell")
	UBoxComponent* OxygenVolume;

	// -- PROPERTIES --
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Oxygen")
	float MaxOxygen = 100.0f;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Oxygen")
	float CurrentOxygen;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Oxygen", meta = (ClampMin = "0.0", ClampMax = "10.0"))
	float OxygenDrainRate = 2.0f;
};
