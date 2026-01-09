#pragma once

#include "CoreMinimal.h"
#include "UObject/Interface.h"
#include "InteractableInterface.generated.h"

// This class does not need to be modified.
UINTERFACE(MinimalAPI)
class UInteractableInterface : public UInterface
{
	GENERATED_BODY()
};

/**
 * The Interface Definition.
 * Any actor that implements this (Ship, Loot, Door) must define what happens when "Interacted" with.
 */
class PROJABYSS_API IInteractableInterface
{
	GENERATED_BODY()

public:
	// The function we call when we press E
	// "Instigator" is the person pressing the button (You)
	UFUNCTION(BlueprintNativeEvent, BlueprintCallable, Category = "Interaction")
	void Interact(APawn* InstigatorPawn);
};